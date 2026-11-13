import axios, { AxiosInstance, InternalAxiosRequestConfig } from 'axios';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000';
const API_PREFIX = '/api/v1';

class ApiClient {
  private client: AxiosInstance;
  private accessToken: string | null = null;
  private refreshToken: string | null = null;

  constructor() {
    this.client = axios.create({
      baseURL: `${API_BASE_URL}${API_PREFIX}`,
      headers: {
        'Content-Type': 'application/json',
      },
      withCredentials: true,
    });

    // Request interceptor to add auth token
    this.client.interceptors.request.use(
      (config: InternalAxiosRequestConfig) => {
        if (this.accessToken) {
          config.headers.Authorization = `Bearer ${this.accessToken}`;
        }
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor to handle token refresh
    this.client.interceptors.response.use(
      (response) => response,
      async (error) => {
        const originalRequest = error.config;
        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true;
          try {
            await this.refreshAccessToken();
            originalRequest.headers.Authorization = `Bearer ${this.accessToken}`;
            return this.client(originalRequest);
          } catch (refreshError) {
            this.clearTokens();
            if (typeof window !== 'undefined') {
              window.location.href = '/login';
            }
            return Promise.reject(refreshError);
          }
        }
        return Promise.reject(error);
      }
    );

    // Load tokens from localStorage on init
    if (typeof window !== 'undefined') {
      this.accessToken = localStorage.getItem('access_token');
      this.refreshToken = localStorage.getItem('refresh_token');
    }
  }

  setTokens(access: string, refresh: string) {
    this.accessToken = access;
    this.refreshToken = refresh;
    if (typeof window !== 'undefined') {
      localStorage.setItem('access_token', access);
      localStorage.setItem('refresh_token', refresh);
    }
  }

  clearTokens() {
    this.accessToken = null;
    this.refreshToken = null;
    if (typeof window !== 'undefined') {
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
    }
  }

  async refreshAccessToken(): Promise<void> {
    if (!this.refreshToken) throw new Error('No refresh token');
    const response = await axios.post(
      `${API_BASE_URL}${API_PREFIX}/auth/refresh`,
      { refresh_token: this.refreshToken },
      { headers: { 'Content-Type': 'application/json' } }
    );
    this.setTokens(response.data.access_token, response.data.refresh_token);
  }

  // Auth endpoints
  async register(username: string, email: string, password: string) {
    const response = await this.client.post('/auth/register', { username, email, password });
    this.setTokens(response.data.access_token, response.data.refresh_token);
    return response.data;
  }

  async login(username: string, password: string) {
    const response = await this.client.post('/auth/login', { username, password });
    this.setTokens(response.data.access_token, response.data.refresh_token);
    return response.data;
  }

  async logout() {
    try {
      await this.client.post('/auth/logout');
    } finally {
      this.clearTokens();
    }
  }

  async getCurrentUser() {
    const response = await this.client.get('/auth/me');
    return response.data;
  }

  async changePassword(currentPassword: string, newPassword: string) {
    const response = await this.client.post('/auth/change-password', { current_password: currentPassword, new_password: newPassword });
    return response.data;
  }

  // File endpoints
  async listFiles(parentId?: number, page = 1, pageSize = 20) {
    const params = new URLSearchParams();
    if (parentId) params.append('parent_id', parentId.toString());
    params.append('page', page.toString());
    params.append('page_size', pageSize.toString());
    const response = await this.client.get(`/files?${params.toString()}`);
    return response.data;
  }

  async createFolder(name: string, parentId?: number) {
    const response = await this.client.post('/files', { name, parent_id: parentId });
    return response.data;
  }

  async deleteFile(fileId: number) {
    const response = await this.client.delete(`/files/${fileId}`);
    return response.data;
  }

  // Upload endpoints
  async initiateUpload(filename: string, sizeBytes: number, mimeType: string, parentId?: number) {
    const response = await this.client.post('/uploads/initiate', {
      filename,
      size_bytes: sizeBytes,
      mime_type: mimeType,
      parent_id: parentId,
    });
    return response.data;
  }

  async uploadChunk(uploadId: string, chunkNumber: number, chunk: Blob) {
    const formData = new FormData();
    formData.append('file', chunk);
    const response = await this.client.put(`/uploads/${uploadId}/chunks/${chunkNumber}`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return response.data;
  }

  async getUploadStatus(uploadId: string) {
    const response = await this.client.get(`/uploads/${uploadId}/status`);
    return response.data;
  }

  async completeUpload(uploadId: string) {
    const response = await this.client.post(`/uploads/${uploadId}/complete`);
    return response.data;
  }

  // Share endpoints
  async createShare(fileId: number, recipientEmail?: string, permissions = 'read', expiresAt?: string) {
    const response = await this.client.post('/shares', {
      file_id: fileId,
      recipient_email: recipientEmail,
      permissions,
      expires_at: expiresAt,
    });
    return response.data;
  }
}

export const api = new ApiClient();