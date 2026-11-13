'use client';

import { useState } from 'react';
import { useFileStore, useUploadStore } from '@/lib/store';
import { api } from '@/lib/api';
import { Folder, File, Upload, Download, Trash2, Share2, MoreVertical, ChevronRight, Loader2 } from 'lucide-react';
import toast from 'react-hot-toast';

interface FileItem {
  id: number;
  name: string;
  mime_type: string;
  size_bytes: number;
  is_folder: boolean;
  created_at: string;
  parent_id: number | null;
}

export default function FileList() {
  const { files, currentFolder, setFiles, setCurrentFolder, addFile, removeFile, setLoading, breadcrumbs, pagination } = useFileStore();
  const { uploads, addUpload, updateUpload, removeUpload } = useUploadStore();
  const [selectedFiles, setSelectedFiles] = useState<Set<number>>(new Set());
  const [uploading, setUploading] = useState(false);

  const formatSize = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getFileIcon = (mimeType: string, isFolder: boolean) => {
    if (isFolder) return <Folder className="w-6 h-6 text-yellow-500" />;
    if (mimeType.startsWith('image/')) return <File className="w-6 h-6 text-green-500" />;
    if (mimeType.startsWith('video/')) return <File className="w-6 h-6 text-red-500" />;
    if (mimeType.startsWith('audio/')) return <File className="w-6 h-6 text-orange-500" />;
    if (mimeType === 'application/pdf') return <File className="w-6 h-6 text-red-600" />;
    return <File className="w-6 h-6 text-gray-500" />;
  };

  const loadFiles = async () => {
    setLoading(true);
    try {
      const response = await api.listFiles(currentFolder, pagination.page, pagination.pageSize);
      setFiles(response);
    } catch (error) {
      toast.error('Failed to load files');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateFolder = async () => {
    const name = prompt('Enter folder name:');
    if (!name) return;
    try {
      const folder = await api.createFolder(name, currentFolder);
      addFile(folder);
      toast.success('Folder created');
    } catch (error) {
      toast.error('Failed to create folder');
    }
  };

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setUploading(true);
    try {
      // Initiate upload
      const uploadData = await api.initiateUpload(file.name, file.size, file.type, currentFolder);
      const { upload_id, chunk_size_bytes, total_chunks } = uploadData;

      const session = {
        uploadId: upload_id,
        file: file,
        chunks: [],
        totalChunks: total_chunks,
        uploadedChunks: 0,
        status: 'uploading' as const,
        progress: 0,
      };

      // Split file into chunks
      for (let i = 0; i < total_chunks; i++) {
        const start = i * chunk_size_bytes;
        const end = Math.min(start + chunk_size_bytes, file.size);
        session.chunks.push(file.slice(start, end));
      }

      addUpload(session);

      // Upload chunks
      for (let i = 0; i < total_chunks; i++) {
        await api.uploadChunk(upload_id, i + 1, session.chunks[i]);
        updateUpload(upload_id, {
          uploadedChunks: i + 1,
          progress: ((i + 1) / total_chunks) * 100,
        });
      }

      // Complete upload
      const completedFile = await api.completeUpload(upload_id);
      addFile(completedFile);
      removeUpload(upload_id);
      toast.success(`${file.name} uploaded successfully`);
    } catch (error) {
      toast.error('Upload failed');
    } finally {
      setUploading(false);
      event.currentTarget.value = '';
    }
  };

  const handleDelete = async (fileId: number) => {
    if (!confirm('Delete this file?')) return;
    try {
      await api.deleteFile(fileId);
      removeFile(fileId);
      toast.success('File deleted');
    } catch (error) {
      toast.error('Failed to delete file');
    }
  };

  const handleNavigate = (file: FileItem) => {
    if (file.is_folder) {
      setCurrentFolder(file.id, file.name);
      loadFiles();
    }
  };

  const handleBreadcrumbClick = (index: number) => {
    const crumb = breadcrumbs[index];
    if (crumb.id === currentFolder) return;
    setCurrentFolder(crumb.id, crumb.name);
    loadFiles();
  };

  return (
    <div className="flex flex-col h-full">
      {/* Breadcrumbs */}
      <div className="px-4 py-2 border-b bg-gray-50 flex items-center gap-1 overflow-x-auto">
        {breadcrumbs.map((crumb, index) => (
          <React.Fragment key={crumb.id ?? 'root'}>
            <button
              onClick={() => handleBreadcrumbClick(index)}
              className={`text-sm px-2 py-1 rounded transition-colors ${
                crumb.id === currentFolder
                  ? 'bg-primary-100 text-primary-700 font-medium'
                  : 'text-gray-600 hover:bg-gray-100'
              }`}
            >
              {crumb.name}
            </button>
            {index < breadcrumbs.length - 1 && <ChevronRight className="w-4 h-4 text-gray-400" />}
          </React.Fragment>
        ))}
      </div>

      {/* Toolbar */}
      <div className="px-4 py-3 border-b bg-white flex items-center gap-3">
        <button
          onClick={handleCreateFolder}
          className="px-3 py-1.5 bg-primary-600 text-white text-sm rounded-lg hover:bg-primary-700 transition-colors flex items-center gap-2"
        >
          <Folder className="w-4 h-4" /> New Folder
        </button>
        <label className="px-3 py-1.5 bg-gray-100 text-gray-700 text-sm rounded-lg hover:bg-gray-200 transition-colors cursor-pointer flex items-center gap-2">
          <Upload className="w-4 h-4" /> Upload Files
          <input
            type="file"
            multiple
            onChange={handleFileUpload}
            className="sr-only"
            disabled={uploading}
          />
        </label>

        <div className="flex-1" />

        {selectedFiles.size > 0 && (
          <button
            onClick={() => {
              selectedFiles.forEach(id => handleDelete(id));
              setSelectedFiles(new Set());
            }}
            className="px-3 py-1.5 bg-red-600 text-white text-sm rounded-lg hover:bg-red-700 transition-colors"
          >
            <Trash2 className="w-4 h-4" /> Delete ({selectedFiles.size})
          </button>
        )}
      </div>

      {/* Upload Progress */}
      {uploads.size > 0 && (
        <div className="px-4 py-2 border-b bg-gray-50">
          {Array.from(uploads.values()).map((upload) => (
            <div key={upload.uploadId} className="mb-2">
              <div className="flex items-center justify-between text-sm mb-1">
                <span className="truncate max-w-[200px]">{upload.file.name}</span>
                <span>{Math.round(upload.progress)}%</span>
              </div>
              <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary-600 transition-all duration-300"
                  style={{ width: `${upload.progress}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      )}

      {/* File List */}
      <div className="flex-1 overflow-auto">
        {files.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-500">
            <Folder className="w-16 h-16 mb-4 text-gray-300" />
            <p>This folder is empty</p>
            <p className="text-sm">Upload files or create a folder to get started</p>
          </div>
        ) : (
          <div className="divide-y">
            {files.map((file) => (
              <div
                key={file.id}
                className={`flex items-center px-4 py-3 hover:bg-gray-50 transition-colors ${
                  selectedFiles.has(file.id) ? 'bg-primary-50' : ''
                }`}
                onClick={() => handleNavigate(file)}
                onDoubleClick={() => handleNavigate(file)}
              >
                <input
                  type="checkbox"
                  checked={selectedFiles.has(file.id)}
                  onChange={(e) => {
                    e.stopPropagation();
                    const newSet = new Set(selectedFiles);
                    if (e.target.checked) newSet.add(file.id);
                    else newSet.delete(file.id);
                    setSelectedFiles(newSet);
                  }}
                  className="w-4 h-4 text-primary-600 rounded border-gray-300 mr-3"
                />
                {getFileIcon(file.mime_type, file.is_folder)}
                <div className="flex-1 min-w-0 ml-3">
                  <p className="font-medium truncate">{file.name}</p>
                  <p className="text-xs text-gray-500">
                    {!file.is_folder && formatSize(file.size_bytes)}
                    {file.is_folder && 'Folder'}
                    {' • '}
                    {new Date(file.created_at).toLocaleDateString()}
                  </p>
                </div>
                <div className="flex items-center gap-2 text-gray-400">
                  {!file.is_folder && (
                    <button className="p-1.5 hover:bg-gray-100 rounded" title="Download">
                      <Download className="w-4 h-4" />
                    </button>
                  )}
                  <button className="p-1.5 hover:bg-gray-100 rounded" title="Share">
                    <Share2 className="w-4 h-4" />
                  </button>
                  <button className="p-1.5 hover:bg-gray-100 rounded" title="More">
                    <MoreVertical className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Pagination */}
        {pagination.totalPages > 1 && (
          <div className="px-4 py-3 border-t flex items-center justify-between">
            <span className="text-sm text-gray-600">
              Page {pagination.page} of {pagination.totalPages} ({pagination.total} items)
            </span>
            <div className="flex gap-2">
              <button
                onClick={() => {
                  if (pagination.page > 1) {
                    // Reload with new page
                  }
                }}
                disabled={pagination.page === 1}
                className="px-3 py-1 text-sm border rounded disabled:opacity-50"
              >
                Previous
              </button>
              <button
                onClick={() => {
                  if (pagination.page < pagination.totalPages) {
                    // Reload with new page
                  }
                }}
                disabled={pagination.page === pagination.totalPages}
                className="px-3 py-1 text-sm border rounded disabled:opacity-50"
              >
                Next
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}