import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface User {
  id: number;
  username: string;
  email: string;
  role: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

interface File {
  id: number;
  owner_id: number;
  parent_id: number | null;
  name: string;
  storage_key: string;
  mime_type: string;
  size_bytes: number;
  checksum: string;
  version: number;
  is_folder: boolean;
  created_at: string;
  updated_at: string;
}

interface FileListResponse {
  items: File[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

interface UploadSession {
  uploadId: string;
  file: File;
  chunks: Blob[];
  totalChunks: number;
  uploadedChunks: number;
  status: 'pending' | 'uploading' | 'completed' | 'error';
  progress: number;
}

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (user: User) => void;
  logout: () => void;
  setUser: (user: User | null) => void;
  setLoading: (loading: boolean) => void;
}

interface FileState {
  files: File[];
  currentFolder: number | null;
  breadcrumbs: Array<{ id: number | null; name: string }>;
  isLoading: boolean;
  pagination: {
    page: number;
    pageSize: number;
    total: number;
    totalPages: number;
  };
  setFiles: (files: FileListResponse) => void;
  setCurrentFolder: (folderId: number | null, folderName?: string) => void;
  addFile: (file: File) => void;
  removeFile: (fileId: number) => void;
  setLoading: (loading: boolean) => void;
  loadFiles: (page?: number, pageSize?: number) => Promise<void>;
}

interface UploadState {
  uploads: Map<string, UploadSession>;
  addUpload: (upload: UploadSession) => void;
  updateUpload: (uploadId: string, updates: Partial<UploadSession>) => void;
  removeUpload: (uploadId: string) => void;
  clearUploads: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      isAuthenticated: false,
      isLoading: true,
      login: (user) => set({ user, isAuthenticated: true }),
      logout: () => set({ user: null, isAuthenticated: false }),
      setUser: (user) => set({ user, isAuthenticated: !!user }),
      setLoading: (isLoading) => set({ isLoading }),
    }),
    {
      name: 'ar3mis-auth',
      partialize: (state) => ({ user: state.user, isAuthenticated: state.isAuthenticated }),
    }
  )
);

export const useFileStore = create<FileState>((set, get) => ({
  files: [],
  currentFolder: null,
  breadcrumbs: [{ id: null, name: 'Root' }],
  isLoading: false,
  pagination: { page: 1, pageSize: 20, total: 0, totalPages: 0 },
  setFiles: (response) =>
    set({
      files: response.items,
      pagination: {
        page: response.page,
        pageSize: response.page_size,
        total: response.total,
        totalPages: response.total_pages,
      },
      isLoading: false,
    }),
  setCurrentFolder: (folderId, folderName) =>
    set((state) => {
      let newBreadcrumbs = state.breadcrumbs;
      if (folderId === null) {
        newBreadcrumbs = [{ id: null, name: 'Root' }];
      } else if (folderName) {
        const existing = state.breadcrumbs.find((b) => b.id === folderId);
        if (!existing) {
          newBreadcrumbs = [...state.breadcrumbs, { id: folderId, name: folderName }];
        }
      }
      return { currentFolder: folderId, breadcrumbs: newBreadcrumbs };
    }),
  addFile: (file) =>
    set((state) => ({ files: [file, ...state.files] })),
  removeFile: (fileId) =>
    set((state) => ({ files: state.files.filter((f) => f.id !== fileId) })),
  setLoading: (isLoading) => set({ isLoading }),
  loadFiles: async (page = 1, pageSize = 20) => {
    set({ isLoading: true });
    try {
      const { api } = await import('@/lib/api');
      const response = await api.listFiles(get().currentFolder, page, pageSize);
      get().setFiles(response);
    } catch (error) {
      set({ isLoading: false });
      throw error;
    }
  },
}));

export const useUploadStore = create<UploadState>((set) => ({
  uploads: new Map(),
  addUpload: (upload) =>
    set((state) => {
      const newUploads = new Map(state.uploads);
      newUploads.set(upload.uploadId, upload);
      return { uploads: newUploads };
    }),
  updateUpload: (uploadId, updates) =>
    set((state) => {
      const newUploads = new Map(state.uploads);
      const existing = newUploads.get(uploadId);
      if (existing) {
        newUploads.set(uploadId, { ...existing, ...updates });
      }
      return { uploads: newUploads };
    }),
  removeUpload: (uploadId) =>
    set((state) => {
      const newUploads = new Map(state.uploads);
      newUploads.delete(uploadId);
      return { uploads: newUploads };
    }),
  clearUploads: () => set({ uploads: new Map() }),
}));