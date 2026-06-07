import { useEffect, useCallback, useState } from 'react';
import { X, ZoomIn, ZoomOut } from 'lucide-react';

interface ImagePreviewModalProps {
  /** 图片 URL（data URI 或 远程地址） */
  src: string;
  /** 可选：文件名 */
  filename?: string;
  /** 可选：MIME 类型 */
  mediaType?: string;
  /** 可选：文件大小（字节） */
  sizeBytes?: number;
  /** 关闭回调 */
  onClose: () => void;
}

/**
 * 全屏图片预览 Modal
 * - 暗色遮罩 + 毛玻璃背景
 * - ESC 或点击背景关闭
 * - 缩放控制 (0.5x ~ 3x)
 * - 顶部工具栏 + 底部信息栏
 */
export function ImagePreviewModal({
  src,
  filename,
  mediaType,
  sizeBytes,
  onClose,
}: ImagePreviewModalProps) {
  const [scale, setScale] = useState(1);

  // ESC 键关闭
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [onClose]);

  // 阻止背景滚动
  useEffect(() => {
    document.body.style.overflow = 'hidden';
    return () => { document.body.style.overflow = ''; };
  }, []);

  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) onClose();
    },
    [onClose],
  );

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-fade-in"
      onClick={handleBackdropClick}
    >
      {/* 顶部工具栏 */}
      <div className="absolute top-0 left-0 right-0 flex items-center justify-between px-4 py-3 bg-gradient-to-b from-black/40 to-transparent z-10">
        <span className="text-sm text-white/80 truncate max-w-[60%]">
          {filename || '图片预览'}
        </span>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setScale((s) => Math.max(0.5, s - 0.25))}
            className="p-1.5 rounded-lg bg-white/10 text-white hover:bg-white/20 transition-colors"
            title="缩小"
          >
            <ZoomOut className="w-4 h-4" />
          </button>
          <span className="text-xs text-white/60 w-8 text-center">{Math.round(scale * 100)}%</span>
          <button
            onClick={() => setScale((s) => Math.min(3, s + 0.25))}
            className="p-1.5 rounded-lg bg-white/10 text-white hover:bg-white/20 transition-colors"
            title="放大"
          >
            <ZoomIn className="w-4 h-4" />
          </button>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg bg-white/10 text-white hover:bg-red-400/70 transition-colors ml-2"
            title="关闭"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* 图片 */}
      <div className="p-8 max-w-full max-h-full flex items-center justify-center overflow-hidden">
        <img
          src={src}
          alt={filename || '预览'}
          className="max-w-full max-h-[85vh] object-contain rounded-lg shadow-2xl transition-transform duration-200 ease-out select-none"
          style={{ transform: `scale(${scale})`, cursor: scale > 1 ? 'grab' : 'default' }}
          draggable={false}
        />
      </div>

      {/* 底部信息 */}
      <div className="absolute bottom-0 left-0 right-0 flex items-center justify-center gap-4 px-4 py-3 bg-gradient-to-t from-black/40 to-transparent">
        {sizeBytes !== undefined && (
          <span className="text-xs text-white/60">
            {(sizeBytes / 1024).toFixed(1)} KB
          </span>
        )}
        {mediaType && (
          <span className="text-xs text-white/60">{mediaType}</span>
        )}
      </div>
    </div>
  );
}
