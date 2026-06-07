import { useState, useRef, useCallback, useEffect, type DragEvent, type ClipboardEvent } from 'react';
import { Image, Paperclip, Send, Square, X } from 'lucide-react';
import type { FileAttachment as ChatFileAttachment } from '../../types/chat';

interface ChatInputProps {
  onSend: (text: string, images?: string[], files?: ChatFileAttachment[]) => void;
  onCancel: () => void;
  isStreaming: boolean;
  disabled?: boolean;
}

/* ---- 本地文件预览 ---- */
interface LocalImage {
  id: string;
  file: File;
  /** 预览 URL (createObjectURL) */
  previewUrl: string;
}

interface LocalFile {
  id: string;
  name: string;
  size: number;
}

const MAX_FILE_SIZE = 20 * 1024 * 1024; // 20MB

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function genId(): string {
  return `file_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

export function ChatInput({
  onSend,
  onCancel,
  isStreaming,
  disabled,
}: ChatInputProps) {
  const [text, setText] = useState('');
  const [localImages, setLocalImages] = useState<LocalImage[]>([]);
  const [localFiles, setLocalFiles] = useState<LocalFile[]>([]);
  const [dragOver, setDragOver] = useState(false);

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const imageInputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  /** 自动增高 */
  const adjustHeight = useCallback(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${Math.min(el.scrollHeight, 200)}px`;
  }, []);

  /** 清除附件 */
  const clearAttachments = useCallback(() => {
    // 释放对象 URL
    localImages.forEach((img) => URL.revokeObjectURL(img.previewUrl));
    setLocalImages([]);
    setLocalFiles([]);
  }, [localImages]);

  /** 读取 File 为 base64 (不含前缀) */
  const fileToBase64 = useCallback((file: File): Promise<string> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result as string;
        const comma = result.indexOf(',');
        resolve(comma >= 0 ? result.slice(comma + 1) : result);
      };
      reader.onerror = () => reject(new Error(`读取文件失败: ${file.name}`));
      reader.readAsDataURL(file);
    });
  }, []);

  /** 发送 */
  const handleSend = useCallback(async () => {
    const trimmed = text.trim();
    if (!trimmed && localImages.length === 0 && localFiles.length === 0) return;
    if (isStreaming) return;

    // 图片 → base64
    const imageB64s: string[] = [];
    for (const img of localImages) {
      try {
        const b64 = await fileToBase64(img.file);
        imageB64s.push(b64);
      } catch {
        // 跳过读取失败的
      }
    }

    // 文件 → ChatFileAttachment
    const attachments: ChatFileAttachment[] = [];
    for (const f of localFiles) {
      // 本地文件未存储实际数据，发送时传空 data；由 user 自行决定
      // 实际项目中应读取为 base64
      attachments.push({
        id: f.id,
        name: f.name,
        mimeType: 'application/octet-stream',
        sizeBytes: f.size,
        data: '',
      });
    }

    onSend(trimmed, imageB64s.length > 0 ? imageB64s : undefined, attachments.length > 0 ? attachments : undefined);
    setText('');
    clearAttachments();
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  }, [text, localImages, localFiles, isStreaming, onSend, clearAttachments, fileToBase64]);

  /** 键盘快捷键 */
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  /* ========== 添加图片 ========== */
  const addImagesFromFiles = useCallback((fileList: FileList | File[]) => {
    const newImages: LocalImage[] = [];
    for (const file of Array.from(fileList)) {
      if (!file.type.startsWith('image/')) continue;
      if (file.size > MAX_FILE_SIZE) continue;
      newImages.push({
        id: genId(),
        file,
        previewUrl: URL.createObjectURL(file),
      });
    }
    if (newImages.length > 0) {
      setLocalImages((prev) => [...prev, ...newImages]);
    }
  }, []);

  const addFilesFromFiles = useCallback((fileList: FileList | File[]) => {
    const newFiles: LocalFile[] = [];
    for (const file of Array.from(fileList)) {
      if (file.size > MAX_FILE_SIZE) continue;
      newFiles.push({ id: genId(), name: file.name, size: file.size });
    }
    if (newFiles.length > 0) {
      setLocalFiles((prev) => [...prev, ...newFiles]);
    }
  }, []);

  const removeLocalImage = useCallback((id: string) => {
    setLocalImages((prev) => {
      const img = prev.find((i) => i.id === id);
      if (img) URL.revokeObjectURL(img.previewUrl);
      return prev.filter((i) => i.id !== id);
    });
  }, []);

  const removeLocalFile = useCallback((id: string) => {
    setLocalFiles((prev) => prev.filter((f) => f.id !== id));
  }, []);

  /* ========== 拖拽 ========== */
  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);
  }, []);

  const handleDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setDragOver(false);

      const files = Array.from(e.dataTransfer.files);
      const imageFiles = files.filter((f) => f.type.startsWith('image/'));
      const otherFiles = files.filter((f) => !f.type.startsWith('image/'));

      if (imageFiles.length > 0) addImagesFromFiles(imageFiles);
      if (otherFiles.length > 0) addFilesFromFiles(otherFiles);
    },
    [addImagesFromFiles, addFilesFromFiles],
  );

  /* ========== 粘贴 ========== */
  const handlePaste = useCallback(
    (e: ClipboardEvent) => {
      const items = Array.from(e.clipboardData.items);
      const imageFiles: File[] = [];
      for (const item of items) {
        if (item.kind !== 'file') continue;
        const file = item.getAsFile();
        if (!file || !file.type.startsWith('image/')) continue;
        if (file.size > MAX_FILE_SIZE) continue;
        imageFiles.push(file);
      }
      if (imageFiles.length > 0) {
        e.preventDefault();
        addImagesFromFiles(imageFiles);
      }
    },
    [addImagesFromFiles],
  );

  /* ========== 文件选择器 ========== */
  const handleImageInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files) {
        addImagesFromFiles(e.target.files);
        e.target.value = '';
      }
    },
    [addImagesFromFiles],
  );

  const handleFileInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files) {
        addFilesFromFiles(e.target.files);
        e.target.value = '';
      }
    },
    [addFilesFromFiles],
  );

  /* ---- 组件卸载时清理对象 URL ---- */
  useEffect(() => {
    return () => {
      localImages.forEach((img) => URL.revokeObjectURL(img.previewUrl));
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const hasContent = text.trim().length > 0 || localImages.length > 0 || localFiles.length > 0;

  return (
    <div className="flex-shrink-0 px-4 pb-4 pt-2">
      <div
        className={`
          relative rounded-2xl transition-all duration-200
          ${dragOver
            ? 'border-2 border-dashed border-indigo-400/60 bg-indigo-50/50 dark:bg-indigo-950/30'
            : 'glass shadow-lg shadow-black/5 dark:shadow-black/20'
          }
          focus-within:ring-2 focus-within:ring-indigo-500/30 focus-within:border-indigo-500/40
        `}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        {/* 拖拽提示浮层 */}
        {dragOver && (
          <div className="absolute inset-0 rounded-2xl flex items-center justify-center z-10 pointer-events-none">
            <div className="text-center">
              <Image className="w-8 h-8 mx-auto mb-2 text-indigo-400" />
              <p className="text-sm font-medium text-indigo-500 dark:text-indigo-400">
                松开以上传文件
              </p>
            </div>
          </div>
        )}

        {/* 图片预览缩略图 */}
        {localImages.length > 0 && (
          <div className="flex flex-wrap gap-2 px-4 pt-3">
            {localImages.map((img) => (
              <div
                key={img.id}
                className="relative group rounded-xl overflow-hidden border border-white/20 dark:border-gray-600/40 bg-white/30 dark:bg-gray-800/30 backdrop-blur-sm flex-shrink-0"
                style={{ width: '72px', height: '72px' }}
              >
                <img
                  src={img.previewUrl}
                  alt={img.file.name}
                  className="w-full h-full object-cover"
                />
                <button
                  onClick={() => removeLocalImage(img.id)}
                  className="absolute top-1 right-1 w-5 h-5 rounded-full bg-black/50 backdrop-blur-sm text-white flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-red-500/70 transition-all duration-200"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        )}

        {/* 文件列表 */}
        {localFiles.length > 0 && (
          <div className="flex flex-wrap gap-2 px-4 pt-3">
            {localFiles.map((f) => (
              <div
                key={f.id}
                className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/30 dark:bg-gray-800/40 backdrop-blur-sm border border-white/20 dark:border-gray-600/30 text-xs"
              >
                <Paperclip className="w-3.5 h-3.5 text-indigo-400" />
                <span className="text-gray-700 dark:text-gray-300 max-w-[120px] truncate">{f.name}</span>
                <span className="text-gray-400 dark:text-gray-500">{formatSize(f.size)}</span>
                <button
                  onClick={() => removeLocalFile(f.id)}
                  className="text-gray-400 hover:text-red-400 transition-colors"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        )}

        {/* 输入区域 */}
        <div className="flex items-end gap-2 px-4 py-3">
          {/* 图片上传按钮 */}
          <button
            onClick={() => imageInputRef.current?.click()}
            disabled={disabled || isStreaming}
            className="flex-shrink-0 p-2 rounded-xl text-gray-400 dark:text-gray-500 hover:bg-gray-100/50 dark:hover:bg-gray-800/50 hover:text-indigo-500 dark:hover:text-indigo-400 disabled:opacity-40 transition-all duration-200"
            title="上传图片"
          >
            <Image className="w-5 h-5" />
          </button>
          <input
            ref={imageInputRef}
            type="file"
            accept="image/*"
            multiple
            className="hidden"
            onChange={handleImageInputChange}
          />

          {/* 文件上传按钮 */}
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={disabled || isStreaming}
            className="flex-shrink-0 p-2 rounded-xl text-gray-400 dark:text-gray-500 hover:bg-gray-100/50 dark:hover:bg-gray-800/50 hover:text-indigo-500 dark:hover:text-indigo-400 disabled:opacity-40 transition-all duration-200"
            title="上传文件"
          >
            <Paperclip className="w-5 h-5" />
          </button>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            className="hidden"
            onChange={handleFileInputChange}
          />

          {/* 文本框 */}
          <textarea
            ref={textareaRef}
            value={text}
            onChange={(e) => {
              setText(e.target.value);
              adjustHeight();
            }}
            onKeyDown={handleKeyDown}
            onPaste={handlePaste}
            placeholder={disabled ? '请先选择或新建一个对话' : '输入消息... (Enter 发送, Shift+Enter 换行)'}
            disabled={disabled || isStreaming}
            rows={1}
            className="flex-1 bg-transparent text-sm text-gray-800 dark:text-gray-200 placeholder:text-gray-400 dark:placeholder:text-gray-500 resize-none outline-none max-h-[200px] disabled:opacity-50"
          />

          {/* 发送 / 停止 */}
          <div className="flex-shrink-0 flex items-center gap-1">
            {isStreaming ? (
              <button
                onClick={onCancel}
                className="p-2 rounded-xl bg-red-500/10 text-red-500 hover:bg-red-500/20 transition-all duration-200"
                title="停止生成"
              >
                <Square className="w-5 h-5" />
              </button>
            ) : (
              <button
                onClick={handleSend}
                disabled={disabled || !hasContent}
                className="p-2 rounded-xl bg-gradient-to-r from-indigo-500 to-purple-500 text-white shadow-lg shadow-indigo-500/20 hover:from-indigo-600 hover:to-purple-600 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed transition-all duration-200"
                title="发送"
              >
                <Send className="w-5 h-5" />
              </button>
            )}
          </div>
        </div>

        {/* 底部提示 */}
        {!isStreaming && !disabled && (
          <div className="px-4 pb-2 flex items-center gap-3 text-[10px] text-gray-400 dark:text-gray-500">
            <span>拖拽文件到此处</span>
            <span>·</span>
            <span>支持图片和文档</span>
          </div>
        )}
      </div>
    </div>
  );
}
