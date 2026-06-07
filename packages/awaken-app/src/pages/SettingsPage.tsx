import { useState, useEffect, useCallback } from 'react';
import {
  Save,
  RotateCcw,
  Eye,
  EyeOff,
  ChevronDown,
  CheckCircle2,
  AlertCircle,
  Settings2,
  Mail,
  MessageSquare,
  Bot,
  Globe,
  Send,
  Smartphone,
  RefreshCw,
  Plus,
} from 'lucide-react';

/* ============================================================
   类型定义
   ============================================================ */
interface AgentConfig {
  agentId: string;
  model: string;
  provider: string;
  temperature: number;
  maxTokens: number;
  systemPrompt: string;
  visionProvider: string;
  visionModel: string;
  visionApiKey: string;
  visionBaseUrl: string;
  maxImageSize: number;
  email: { smtpHost: string; smtpPort: number; username: string; password: string; fromAddress: string };
  dingtalk: { webhookUrl: string; secret: string };
  wecom: { webhookUrl: string };
  feishu: { webhookUrl: string; secret: string };
  slack: { webhookUrl: string };
  telegram: { botToken: string; chatId: string };
  opencode: {
    zenApiKey: string;
    autoDiscover: boolean;
    enableCli: boolean;
    cliTimeout: number;
  };
  xfyun: {
    apiKey: string;
    region: string;
    model: string;
    temperature: number;
    maxTokens: number;
    stream: boolean;
    embeddingModel: string;
    rerankModel: string;
    appId: string;
    ttiModel: string;
    ttiWidth: number;
    ttiHeight: number;
    ttiSteps: number;
  };
  mediaGen: {
    openaiApiKey: string;
    agnesApiKey: string;
    defaultImageProvider: string;
    defaultImageModel: string;
    defaultVideoProvider: string;
    defaultVideoModel: string;
    imageSize: string;
  };
}

interface ToastState {
  message: string;
  type: 'success' | 'error';
}

/* ============================================================
   表单常量
   ============================================================ */
const MODELS = [
  'gpt-4o',
  'gpt-4o-mini',
  'claude-3-5-sonnet',
  'claude-3-haiku',
  'llama3',
  'llama3-70b',
  'mistral',
  'mixtral',
  'qwen3.5-2b',
  'agnes-1.5-flash',
  'agnes-2.0-flash',
];
const PROVIDERS = ['openai', 'anthropic', 'ollama', 'xfyun', 'agnes'];
const VISION_PROVIDERS = ['openai', 'anthropic', 'ollama', 'none'];
const XFYUN_REGIONS = ['华北-北京', '华东-上海', '华南-广州'];
const XFYUN_IMAGESIZES = [
  '1024x1024',
  '1792x1024',
  '1024x1792',
  '512x512',
  '768x768',
  '1024x768',
  '768x1024',
];
const IMAGE_PROVIDERS = ['openai', 'agnes'];
const VIDEO_PROVIDERS = ['agnes'];
const IMAGE_MODELS = ['dall-e-3', 'agnes-image-2.0-flash', 'agnes-image-2.1-flash'];
const VIDEO_MODELS = ['agnes-video-v2.0'];
const NOTIFICATION_CHANNELS = ['email', 'dingtalk', 'wecom', 'feishu', 'slack', 'telegram'];

const FREE_MODELS = [
  { id: 'deepseek-v4-flash-free', name: 'DeepSeek V4 Flash Free' },
  { id: 'big-pickle', name: 'Big Pickle Free' },
  { id: 'mimo-v2.5-free', name: 'MiMo V2.5 Free' },
  { id: 'nemotron-3-ultra-free', name: 'Nemotron 3 Ultra Free' },
];

const DEFAULT_CONFIG: AgentConfig = {
  agentId: 'assistant',
  model: 'gpt-4o',
  provider: 'openai',
  temperature: 0.7,
  maxTokens: 2048,
  systemPrompt: 'You are a helpful AI assistant.',
  visionProvider: 'openai',
  visionModel: 'gpt-4o',
  visionApiKey: '',
  visionBaseUrl: '',
  maxImageSize: 20,
  email: { smtpHost: '', smtpPort: 587, username: '', password: '', fromAddress: '' },
  dingtalk: { webhookUrl: '', secret: '' },
  wecom: { webhookUrl: '' },
  feishu: { webhookUrl: '', secret: '' },
  slack: { webhookUrl: '' },
  telegram: { botToken: '', chatId: '' },
  opencode: { zenApiKey: '', autoDiscover: true, enableCli: true, cliTimeout: 300 },
  xfyun: {
    apiKey: '',
    region: '华北-北京',
    model: 'qwen3.5-2b',
    temperature: 0.5,
    maxTokens: 4096,
    stream: false,
    embeddingModel: 'sde0a5839',
    rerankModel: 's125c8e0e',
    appId: '',
    ttiModel: '',
    ttiWidth: 768,
    ttiHeight: 768,
    ttiSteps: 20,
  },
  mediaGen: {
    openaiApiKey: '',
    agnesApiKey: '',
    defaultImageProvider: 'openai',
    defaultImageModel: 'dall-e-3',
    defaultVideoProvider: 'agnes',
    defaultVideoModel: 'agnes-video-v2.0',
    imageSize: '1024x1024',
  },
};

/* ============================================================
   子组件
   ============================================================ */
function TextField({
  label,
  value,
  onChange,
  type = 'text',
  placeholder,
  readOnly,
  rows,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  type?: 'text' | 'password' | 'number' | 'email' | 'url';
  placeholder?: string;
  readOnly?: boolean;
  rows?: number;
}) {
  const [showPassword, setShowPassword] = useState(false);
  const isPassword = type === 'password';
  const effectiveType = isPassword ? (showPassword ? 'text' : 'password') : type;

  return (
    <div className="space-y-1.5">
      <label className="text-sm font-medium text-gray-300">{label}</label>
      <div className="relative">
        {rows ? (
          <textarea
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder={placeholder}
            readOnly={readOnly}
            rows={rows}
            className="w-full px-3 py-2 rounded-xl text-sm bg-gray-800/40 border border-gray-700/30 text-gray-200 placeholder-gray-500 resize-y min-h-[80px] focus:outline-none focus:ring-2 focus:ring-remo-400/40 focus:border-remo-400/60"
          />
        ) : (
          <input
            type={effectiveType}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder={placeholder}
            readOnly={readOnly}
            className="w-full px-3 py-2 rounded-xl text-sm bg-gray-800/40 border border-gray-700/30 text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-remo-400/40 focus:border-remo-400/60"
          />
        )}
        {isPassword ? (
          <button
            type="button"
            onClick={() => setShowPassword((v) => !v)}
            className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded-md text-gray-400 hover:text-gray-200 hover:bg-gray-700/40"
          >
            {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
          </button>
        ) : null}
      </div>
    </div>
  );
}

function SelectField({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: string[];
  onChange: (v: string) => void;
}) {
  return (
    <div className="space-y-1.5">
      <label className="text-sm font-medium text-gray-300">{label}</label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full px-3 py-2 rounded-xl text-sm bg-gray-800/40 border border-gray-700/30 text-gray-200 focus:outline-none focus:ring-2 focus:ring-remo-400/40 focus:border-remo-400/60"
      >
        {options.map((opt) => (
          <option key={opt} value={opt}>
            {opt}
          </option>
        ))}
      </select>
    </div>
  );
}

function SliderField({
  label,
  value,
  min,
  max,
  step,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (v: number) => void;
}) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium text-gray-300">{label}</label>
        <span className="text-xs font-mono text-gray-400 tabular-nums bg-gray-800/50 px-2 py-0.5 rounded-md border border-gray-700/30">
          {value.toFixed(step < 1 ? 1 : 0)}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        className="w-full h-1.5 rounded-full appearance-none cursor-pointer bg-gray-700/50 accent-remo-400 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-gradient-to-r [&::-webkit-slider-thumb]:from-remo-400 [&::-webkit-slider-thumb]:to-remo-500 [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:shadow-remo-500/30 [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:duration-150 [&::-webkit-slider-thumb]:hover:scale-110"
      />
    </div>
  );
}

function Toast({ toast, onClose }: { toast: ToastState | null; onClose: () => void }) {
  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(onClose, 3000);
    return () => clearTimeout(t);
  }, [toast, onClose]);

  if (!toast) return null;

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 animate-fade-in">
      <div
        className={
          'flex items-center gap-2 px-4 py-2.5 rounded-xl text-sm font-medium shadow-lg backdrop-blur-md border ' +
          (toast.type === 'success'
            ? 'bg-emerald-500/15 border-emerald-400/30 text-emerald-200'
            : 'bg-rose-500/15 border-rose-400/30 text-rose-200')
        }
      >
        {toast.type === 'success' ? (
          <CheckCircle2 className="w-4 h-4" />
        ) : (
          <AlertCircle className="w-4 h-4" />
        )}
        <span>{toast.message}</span>
      </div>
    </div>
  );
}

function NotificationChannel({
  id,
  title,
  icon,
  accent,
  open,
  onToggle,
  children,
}: {
  id: string;
  title: string;
  icon: React.ReactNode;
  accent: 'rose' | 'blue' | 'emerald' | 'sky' | 'fuchsia' | 'cyan';
  open: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}) {
  // 静态映射避免 Tailwind 动态 class 丢失
  const accentMap: Record<
    'rose' | 'blue' | 'emerald' | 'sky' | 'fuchsia' | 'cyan',
    { bg: string; border: string; ring: string }
  > = {
    rose: {
      bg: 'bg-gradient-to-br from-rose-400/20 to-rose-500/5',
      border: 'border-rose-400/20',
      ring: 'ring-rose-400/40',
    },
    blue: {
      bg: 'bg-gradient-to-br from-blue-400/20 to-blue-500/5',
      border: 'border-blue-400/20',
      ring: 'ring-blue-400/40',
    },
    emerald: {
      bg: 'bg-gradient-to-br from-emerald-400/20 to-emerald-500/5',
      border: 'border-emerald-400/20',
      ring: 'ring-emerald-400/40',
    },
    sky: {
      bg: 'bg-gradient-to-br from-sky-400/20 to-sky-500/5',
      border: 'border-sky-400/20',
      ring: 'ring-sky-400/40',
    },
    fuchsia: {
      bg: 'bg-gradient-to-br from-fuchsia-400/20 to-fuchsia-500/5',
      border: 'border-fuchsia-400/20',
      ring: 'ring-fuchsia-400/40',
    },
    cyan: {
      bg: 'bg-gradient-to-br from-cyan-400/20 to-cyan-500/5',
      border: 'border-cyan-400/20',
      ring: 'ring-cyan-400/40',
    },
  };
  const colors = accentMap[accent];

  return (
    <div className="glass-card rounded-2xl overflow-hidden">
      <button
        type="button"
        onClick={onToggle}
        className="w-full flex items-center justify-between px-5 py-4 text-left hover:bg-white/[0.02] transition-colors"
      >
        <div className="flex items-center gap-3">
          <div
            className={
              'w-9 h-9 rounded-xl flex items-center justify-center border ' +
              colors.bg +
              ' ' +
              colors.border
            }
          >
            {icon}
          </div>
          <div>
            <h3 className="text-sm font-semibold text-gray-100">{title}</h3>
            <p className="text-xs text-gray-500">配置 {title} 通知</p>
          </div>
        </div>
        <ChevronDown
          className={
            'w-4 h-4 text-gray-400 transition-transform duration-200 ' +
            (open ? 'rotate-180' : '')
          }
        />
      </button>
      {open ? <div className="px-5 pb-5 space-y-3 animate-fade-in">{children}</div> : null}
    </div>
  );
}

/* ============================================================
   主组件
   ============================================================ */
export function SettingsPage() {
  const [config, setConfig] = useState<AgentConfig>(DEFAULT_CONFIG);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [dirty, setDirty] = useState(false);

  const [visionExpanded, setVisionExpanded] = useState(true);
  const [openCodeExpanded, setOpenCodeExpanded] = useState(true);
  const [mediaGenExpanded, setMediaGenExpanded] = useState(false);
  const [xfyunExpanded, setXfyunExpanded] = useState(false);
  const [openNotifications, setOpenNotifications] = useState(true);

  const [openEmail, setOpenEmail] = useState(false);
  const [openDingtalk, setOpenDingtalk] = useState(false);
  const [openWecom, setOpenWecom] = useState(false);
  const [openFeishu, setOpenFeishu] = useState(false);
  const [openSlack, setOpenSlack] = useState(false);
  const [openTelegram, setOpenTelegram] = useState(false);

  const [toast, setToast] = useState<ToastState | null>(null);

  // 通用更新：覆盖整个嵌套对象
  const updateNested = useCallback(
    (
      key: 'opencode' | 'mediaGen' | 'xfyun' | 'email' | 'dingtalk' | 'wecom' | 'feishu' | 'slack' | 'telegram',
      nestedKey: string,
      value: string | number | boolean,
    ) => {
      setConfig((prev) => {
        const current = prev[key] as Record<string, unknown>;
        return {
          ...prev,
          [key]: { ...current, [nestedKey]: value },
        };
      });
      setDirty(true);
    },
    [],
  );

  // 顶层字段更新（任意类型）
  const update = useCallback((patch: Partial<AgentConfig>) => {
    setConfig((prev) => ({ ...prev, ...patch }));
    setDirty(true);
  }, []);

  // 校验
  const validate = useCallback((cfg: AgentConfig): string | null => {
    if (!cfg.agentId.trim()) return 'Agent ID 不能为空';
    if (cfg.maxTokens <= 0) return 'Max Tokens 必须大于 0';
    if (cfg.temperature < 0 || cfg.temperature > 2) return 'Temperature 必须在 0-2 之间';
    if (cfg.maxImageSize <= 0) return 'Max Image Size 必须大于 0';
    if (cfg.opencode.cliTimeout <= 0) return 'CLI Timeout 必须大于 0';
    return null;
  }, []);

  // 加载（从 localStorage 读取以保留之前的修改）
  useEffect(() => {
    setLoading(true);
    try {
      const raw = localStorage.getItem('remo-agent-config');
      if (raw) {
        const parsed = JSON.parse(raw) as AgentConfig;
        setConfig({ ...DEFAULT_CONFIG, ...parsed });
      }
    } catch (e) {
      console.error('读取配置失败', e);
    } finally {
      setLoading(false);
    }
  }, []);

  const handleSave = useCallback(async () => {
    const err = validate(config);
    if (err) {
      setToast({ message: err, type: 'error' });
      return;
    }
    setSaving(true);
    try {
      localStorage.setItem('remo-agent-config', JSON.stringify(config));
      await new Promise((r) => setTimeout(r, 350));
      setDirty(false);
      setToast({ message: '配置已保存', type: 'success' });
    } catch (e) {
      console.error('保存失败', e);
      setToast({ message: '保存失败', type: 'error' });
    } finally {
      setSaving(false);
    }
  }, [config, validate]);

  const handleReset = useCallback(() => {
    setConfig(DEFAULT_CONFIG);
    setDirty(true);
    setToast({ message: '已重置为默认值', type: 'success' });
  }, []);

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="flex items-center gap-2 text-gray-400 text-sm">
          <RefreshCw className="w-4 h-4 animate-spin" />
          加载配置中...
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto px-6 py-6 space-y-5">
      {/* 顶部 Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-gray-100 flex items-center gap-2">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-remo-400/20 to-remo-500/10 border border-remo-400/20 flex items-center justify-center">
              <Settings2 className="w-5 h-5 text-remo-400" />
            </div>
            <span className="gradient-text">Agent 设置</span>
          </h1>
          <p className="text-sm text-gray-500 mt-1.5 ml-11">
            配置 AI Agent 的基础信息、模型、视觉、媒体生成与通知通道
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handleReset}
            disabled={saving}
            className="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-sm font-medium text-gray-300 bg-gray-800/40 border border-gray-700/40 hover:bg-gray-800/70 hover:border-gray-600/60 active:scale-[0.98] transition-all"
          >
            <RotateCcw className="w-4 h-4" />
            重置
          </button>
          <button
            type="button"
            onClick={handleSave}
            disabled={saving || !dirty}
            className="inline-flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium text-white bg-gradient-to-r from-remo-500 to-remo-400 shadow-lg shadow-remo-500/20 hover:from-remo-600 hover:to-remo-500 active:scale-[0.98] transition-all disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {saving ? (
              <RefreshCw className="w-4 h-4 animate-spin" />
            ) : (
              <Save className="w-4 h-4" />
            )}
            {saving ? '保存中' : '保存'}
          </button>
        </div>
      </div>

      {/* 卡片：基本信息（Agent） */}
      <div className="glass-card rounded-2xl overflow-hidden">
        <div className="px-5 py-4 border-b border-white/[0.06]">
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-remo-400" />
            <h2 className="text-base font-semibold text-gray-100">基本信息</h2>
            <span className="text-[10px] px-2 py-0.5 rounded-full bg-remo-400/10 text-remo-400 border border-remo-400/20">
              Agent
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1.5 ml-3.5">Agent 标识、模型与基础生成参数</p>
        </div>
        <div className="p-5 space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <TextField
              label="Agent ID"
              value={config.agentId}
              onChange={(v) => update({ agentId: v })}
              readOnly
            />
            <SelectField
              label="Model"
              value={config.model}
              options={MODELS}
              onChange={(v) => update({ model: v })}
            />
            <SelectField
              label="Provider"
              value={config.provider}
              options={PROVIDERS}
              onChange={(v) => update({ provider: v })}
            />
            <SliderField
              label="Temperature"
              value={config.temperature}
              min={0}
              max={2}
              step={0.1}
              onChange={(v) => update({ temperature: v })}
            />
            <TextField
              label="Max Tokens"
              type="number"
              value={String(config.maxTokens)}
              onChange={(v) => update({ maxTokens: Math.max(1, Number(v) || 0) })}
            />
            <div className="md:col-span-2">
              <TextField
                label="System Prompt"
                value={config.systemPrompt}
                onChange={(v) => update({ systemPrompt: v })}
                rows={4}
                placeholder="You are a helpful AI assistant."
              />
            </div>
          </div>
        </div>
      </div>

      {/* 卡片：Vision 配置（可折叠） */}
      <div className="glass-card rounded-2xl overflow-hidden">
        <button
          type="button"
          onClick={() => setVisionExpanded((v) => !v)}
          className="w-full px-5 py-4 flex items-center justify-between hover:bg-white/[0.02] transition-colors"
        >
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-cyan-400" />
            <h2 className="text-base font-semibold text-gray-100">Vision 配置</h2>
            <span className="text-[10px] px-2 py-0.5 rounded-full bg-cyan-400/10 text-cyan-400 border border-cyan-400/20">
              {config.visionProvider === 'none' ? '已禁用' : config.visionProvider}
            </span>
          </div>
          <ChevronDown
            className={
              'w-4 h-4 text-gray-400 transition-transform duration-200 ' +
              (visionExpanded ? 'rotate-180' : '')
            }
          />
        </button>
        {visionExpanded ? (
          <div className="px-5 pb-5 space-y-4 animate-fade-in">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <SelectField
                label="Vision Provider"
                value={config.visionProvider}
                options={VISION_PROVIDERS}
                onChange={(v) => update({ visionProvider: v })}
              />
              {config.visionProvider !== 'none' ? (
                <>
                  <SelectField
                    label="Vision Model"
                    value={config.visionModel}
                    options={MODELS}
                    onChange={(v) => update({ visionModel: v })}
                  />
                  <TextField
                    label="API Key"
                    type="password"
                    value={config.visionApiKey}
                    onChange={(v) => update({ visionApiKey: v })}
                    placeholder="sk-..."
                  />
                  {config.visionProvider === 'ollama' ? (
                    <TextField
                      label="Base URL"
                      value={config.visionBaseUrl}
                      onChange={(v) => update({ visionBaseUrl: v })}
                      placeholder="http://localhost:11434"
                    />
                  ) : null}
                  <div className="md:col-span-2">
                    <SliderField
                      label="Max Image Size (MB)"
                      value={config.maxImageSize}
                      min={1}
                      max={50}
                      step={1}
                      onChange={(v) => update('maxImageSize', v)}
                    />
                  </div>
                </>
              ) : null}
            </div>
          </div>
        ) : null}
      </div>

      {/* 卡片：OpenCode 配置（可折叠） */}
      <div className="glass-card rounded-2xl overflow-hidden">
        <button
          type="button"
          onClick={() => setOpenCodeExpanded((v) => !v)}
          className="w-full px-5 py-4 flex items-center justify-between hover:bg-white/[0.02] transition-colors"
        >
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
            <h2 className="text-base font-semibold text-gray-100">OpenCode 配置</h2>
            <span className="text-[10px] px-2 py-0.5 rounded-full bg-emerald-400/10 text-emerald-400 border border-emerald-400/20">
              {config.opencode.enableCli ? 'CLI Enabled' : 'CLI Disabled'}
            </span>
          </div>
          <ChevronDown
            className={
              'w-4 h-4 text-gray-400 transition-transform duration-200 ' +
              (openCodeExpanded ? 'rotate-180' : '')
            }
          />
        </button>
        {openCodeExpanded ? (
          <div className="px-5 pb-5 space-y-4 animate-fade-in">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <TextField
                label="Zen API Key"
                type="password"
                value={config.opencode.zenApiKey}
                onChange={(v) => updateNested('opencode', 'zenApiKey', v)}
                placeholder="zen-..."
              />
              <div className="md:col-span-2">
                <label className="text-sm font-medium text-gray-300 mb-2 block">
                  免费模型 (OpenCode Zen)
                </label>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                  {FREE_MODELS.map((m) => (
                    <div
                      key={m.id}
                      className="flex items-center justify-between p-2.5 rounded-xl bg-gray-800/40 border border-gray-700/30"
                    >
                      <div>
                        <p className="text-xs text-gray-200 font-medium">{m.name}</p>
                        <p className="text-[10px] font-mono text-gray-500">{m.id}</p>
                      </div>
                      <span className="text-[10px] px-1.5 py-0.5 rounded-md bg-emerald-500/15 text-emerald-300 border border-emerald-400/20">
                        FREE
                      </span>
                    </div>
                  ))}
                </div>
              </div>
              <div className="flex items-center gap-2 p-3 rounded-xl bg-gray-800/30 border border-gray-700/30">
                <input
                  id="opt-autodiscover"
                  type="checkbox"
                  checked={config.opencode.autoDiscover}
                  onChange={(e) => updateNested('opencode', 'autoDiscover', e.target.checked)}
                  className="w-4 h-4 rounded accent-remo-500"
                />
                <label htmlFor="opt-autodiscover" className="text-sm text-gray-200 cursor-pointer">
                  自动发现本地 OpenCode 实例
                </label>
              </div>
              <div className="flex items-center gap-2 p-3 rounded-xl bg-gray-800/30 border border-gray-700/30">
                <input
                  id="opt-enablecli"
                  type="checkbox"
                  checked={config.opencode.enableCli}
                  onChange={(e) => updateNested('opencode', 'enableCli', e.target.checked)}
                  className="w-4 h-4 rounded accent-remo-500"
                />
                <label htmlFor="opt-enablecli" className="text-sm text-gray-200 cursor-pointer">
                  启用 OpenCode CLI
                </label>
              </div>
              <div className="md:col-span-2">
                <SliderField
                  label="CLI Timeout (s)"
                  value={config.opencode.cliTimeout}
                  min={30}
                  max={1800}
                  step={30}
                  onChange={(v) => updateNested('opencode', 'cliTimeout', v)}
                />
              </div>
            </div>
          </div>
        ) : null}
      </div>

      {/* 卡片：媒体生成配置（可折叠） */}
      <div className="glass-card rounded-2xl overflow-hidden">
        <button
          type="button"
          onClick={() => setMediaGenExpanded((v) => !v)}
          className="w-full px-5 py-4 flex items-center justify-between hover:bg-white/[0.02] transition-colors"
        >
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-fuchsia-400" />
            <h2 className="text-base font-semibold text-gray-100">媒体生成配置</h2>
            <span className="text-[10px] px-2 py-0.5 rounded-full bg-fuchsia-400/10 text-fuchsia-400 border border-fuchsia-400/20">
              Image / Video
            </span>
          </div>
          <ChevronDown
            className={
              'w-4 h-4 text-gray-400 transition-transform duration-200 ' +
              (mediaGenExpanded ? 'rotate-180' : '')
            }
          />
        </button>
        {mediaGenExpanded ? (
          <div className="px-5 pb-5 space-y-4 animate-fade-in">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <TextField
                label="OpenAI API Key"
                type="password"
                value={config.mediaGen.openaiApiKey}
                onChange={(v) => updateNested('mediaGen', 'openaiApiKey', v)}
                placeholder="sk-..."
              />
              <TextField
                label="Agnes API Key"
                type="password"
                value={config.mediaGen.agnesApiKey}
                onChange={(v) => updateNested('mediaGen', 'agnesApiKey', v)}
                placeholder="agnes-..."
              />
              <SelectField
                label="默认图像 Provider"
                value={config.mediaGen.defaultImageProvider}
                options={IMAGE_PROVIDERS}
                onChange={(v) => updateNested('mediaGen', 'defaultImageProvider', v)}
              />
              <SelectField
                label="默认图像 Model"
                value={config.mediaGen.defaultImageModel}
                options={IMAGE_MODELS}
                onChange={(v) => updateNested('mediaGen', 'defaultImageModel', v)}
              />
              <SelectField
                label="默认视频 Provider"
                value={config.mediaGen.defaultVideoProvider}
                options={VIDEO_PROVIDERS}
                onChange={(v) => updateNested('mediaGen', 'defaultVideoProvider', v)}
              />
              <SelectField
                label="默认视频 Model"
                value={config.mediaGen.defaultVideoModel}
                options={VIDEO_MODELS}
                onChange={(v) => updateNested('mediaGen', 'defaultVideoModel', v)}
              />
              <div className="md:col-span-2">
                <SelectField
                  label="默认图像尺寸"
                  value={config.mediaGen.imageSize}
                  options={XFYUN_IMAGESIZES}
                  onChange={(v) => updateNested('mediaGen', 'imageSize', v)}
                />
              </div>
            </div>
          </div>
        ) : null}
      </div>

      {/* 卡片：讯飞星辰 MaaS（可折叠） */}
      <div className="glass-card rounded-2xl overflow-hidden">
        <button
          type="button"
          onClick={() => setXfyunExpanded((v) => !v)}
          className="w-full px-5 py-4 flex items-center justify-between hover:bg-white/[0.02] transition-colors"
        >
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-amber-400" />
            <h2 className="text-base font-semibold text-gray-100">讯飞星辰 MaaS</h2>
            <span className="text-[10px] px-2 py-0.5 rounded-full bg-amber-400/10 text-amber-400 border border-amber-400/20">
              {config.xfyun.region}
            </span>
          </div>
          <ChevronDown
            className={
              'w-4 h-4 text-gray-400 transition-transform duration-200 ' +
              (xfyunExpanded ? 'rotate-180' : '')
            }
          />
        </button>
        {xfyunExpanded ? (
          <div className="px-5 pb-5 space-y-5 animate-fade-in">
            {/* API 配置 */}
            <div>
              <div className="flex items-center gap-2 mb-3">
                <Globe className="w-3.5 h-3.5 text-amber-400" />
                <h3 className="text-sm font-semibold text-gray-200">API 配置</h3>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <TextField
                  label="API Key"
                  type="password"
                  value={config.xfyun.apiKey}
                  onChange={(v) => updateNested('xfyun', 'apiKey', v)}
                  placeholder="讯飞 API Key"
                />
                <SelectField
                  label="Region"
                  value={config.xfyun.region}
                  options={XFYUN_REGIONS}
                  onChange={(v) => updateNested('xfyun', 'region', v)}
                />
                <SelectField
                  label="Model"
                  value={config.xfyun.model}
                  options={MODELS}
                  onChange={(v) => updateNested('xfyun', 'model', v)}
                />
                <TextField
                  label="App ID"
                  value={config.xfyun.appId}
                  onChange={(v) => updateNested('xfyun', 'appId', v)}
                  placeholder="App ID (可选)"
                />
                <SliderField
                  label="Temperature"
                  value={config.xfyun.temperature}
                  min={0}
                  max={2}
                  step={0.1}
                  onChange={(v) => updateNested('xfyun', 'temperature', v)}
                />
                <TextField
                  label="Max Tokens"
                  type="number"
                  value={String(config.xfyun.maxTokens)}
                  onChange={(v) =>
                    updateNested('xfyun', 'maxTokens', Math.max(1, Number(v) || 0))
                  }
                />
                <div className="md:col-span-2 flex items-center gap-2 p-3 rounded-xl bg-gray-800/30 border border-gray-700/30">
                  <input
                    id="opt-xfyun-stream"
                    type="checkbox"
                    checked={config.xfyun.stream}
                    onChange={(e) => updateNested('xfyun', 'stream', e.target.checked)}
                    className="w-4 h-4 rounded accent-remo-500"
                  />
                  <label
                    htmlFor="opt-xfyun-stream"
                    className="text-sm text-gray-200 cursor-pointer"
                  >
                    启用流式响应 (Stream)
                  </label>
                </div>
              </div>
            </div>

            {/* Embedding & Rerank */}
            <div>
              <div className="flex items-center gap-2 mb-3">
                <Bot className="w-3.5 h-3.5 text-amber-400" />
                <h3 className="text-sm font-semibold text-gray-200">Embedding &amp; Rerank</h3>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <TextField
                  label="Embedding Model"
                  value={config.xfyun.embeddingModel}
                  onChange={(v) => updateNested('xfyun', 'embeddingModel', v)}
                />
                <TextField
                  label="Rerank Model"
                  value={config.xfyun.rerankModel}
                  onChange={(v) => updateNested('xfyun', 'rerankModel', v)}
                />
              </div>
            </div>

            {/* TTI 图像生成 */}
            <div>
              <div className="flex items-center gap-2 mb-3">
                <Plus className="w-3.5 h-3.5 text-amber-400" />
                <h3 className="text-sm font-semibold text-gray-200">TTI 图像生成</h3>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <TextField
                  label="TTI Model"
                  value={config.xfyun.ttiModel}
                  onChange={(v) => updateNested('xfyun', 'ttiModel', v)}
                  placeholder="图像生成模型 ID"
                />
                <div />
                <TextField
                  label="Width"
                  type="number"
                  value={String(config.xfyun.ttiWidth)}
                  onChange={(v) =>
                    updateNested('xfyun', 'ttiWidth', Math.max(64, Number(v) || 0))
                  }
                />
                <TextField
                  label="Height"
                  type="number"
                  value={String(config.xfyun.ttiHeight)}
                  onChange={(v) =>
                    updateNested('xfyun', 'ttiHeight', Math.max(64, Number(v) || 0))
                  }
                />
                <div className="md:col-span-2">
                  <SliderField
                    label="Steps"
                    value={config.xfyun.ttiSteps}
                    min={1}
                    max={100}
                    step={1}
                    onChange={(v) => updateNested('xfyun', 'ttiSteps', v)}
                  />
                </div>
              </div>
            </div>
          </div>
        ) : null}
      </div>

      {/* 通知通道 */}
      <div className="glass-card rounded-2xl overflow-hidden">
        <button
          type="button"
          onClick={() => setOpenNotifications((v) => !v)}
          className="w-full px-5 py-4 flex items-center justify-between hover:bg-white/[0.02] transition-colors"
        >
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-rose-400" />
            <h2 className="text-base font-semibold text-gray-100">通知通道</h2>
            <span className="text-[10px] px-2 py-0.5 rounded-full bg-rose-400/10 text-rose-400 border border-rose-400/20">
              {NOTIFICATION_CHANNELS.length} channels
            </span>
          </div>
          <ChevronDown
            className={
              'w-4 h-4 text-gray-400 transition-transform duration-200 ' +
              (openNotifications ? 'rotate-180' : '')
            }
          />
        </button>
        {openNotifications ? (
          <div className="px-5 pb-5 space-y-3 animate-fade-in">
            {/* Email */}
            <NotificationChannel
              id="email"
              title="Email"
              icon={<Mail className="w-4 h-4 text-rose-300" />}
              accent="rose"
              open={openEmail}
              onToggle={() => setOpenEmail((v) => !v)}
            >
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <TextField
                  label="SMTP Host"
                  value={config.email.smtpHost}
                  onChange={(v) => updateNested('email', 'smtpHost', v)}
                  placeholder="smtp.example.com"
                />
                <TextField
                  label="SMTP Port"
                  type="number"
                  value={String(config.email.smtpPort)}
                  onChange={(v) =>
                    updateNested('email', 'smtpPort', Math.max(1, Number(v) || 0))
                  }
                />
                <TextField
                  label="Username"
                  value={config.email.username}
                  onChange={(v) => updateNested('email', 'username', v)}
                />
                <TextField
                  label="Password"
                  type="password"
                  value={config.email.password}
                  onChange={(v) => updateNested('email', 'password', v)}
                />
                <div className="md:col-span-2">
                  <TextField
                    label="From Address"
                    type="email"
                    value={config.email.fromAddress}
                    onChange={(v) => updateNested('email', 'fromAddress', v)}
                    placeholder="noreply@example.com"
                  />
                </div>
              </div>
            </NotificationChannel>

            {/* DingTalk */}
            <NotificationChannel
              id="dingtalk"
              title="DingTalk"
              icon={<MessageSquare className="w-4 h-4 text-blue-300" />}
              accent="blue"
              open={openDingtalk}
              onToggle={() => setOpenDingtalk((v) => !v)}
            >
              <div className="grid grid-cols-1 gap-3">
                <TextField
                  label="Webhook URL"
                  value={config.dingtalk.webhookUrl}
                  onChange={(v) => updateNested('dingtalk', 'webhookUrl', v)}
                  placeholder="https://oapi.dingtalk.com/robot/send?access_token=..."
                />
                <TextField
                  label="Secret"
                  type="password"
                  value={config.dingtalk.secret}
                  onChange={(v) => updateNested('dingtalk', 'secret', v)}
                  placeholder="加签密钥 (可选)"
                />
              </div>
            </NotificationChannel>

            {/* WeCom */}
            <NotificationChannel
              id="wecom"
              title="WeCom"
              icon={<MessageSquare className="w-4 h-4 text-emerald-300" />}
              accent="emerald"
              open={openWecom}
              onToggle={() => setOpenWecom((v) => !v)}
            >
              <TextField
                label="Webhook URL"
                value={config.wecom.webhookUrl}
                onChange={(v) => updateNested('wecom', 'webhookUrl', v)}
                placeholder="https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=..."
              />
            </NotificationChannel>

            {/* Feishu */}
            <NotificationChannel
              id="feishu"
              title="Feishu"
              icon={<Send className="w-4 h-4 text-sky-300" />}
              accent="sky"
              open={openFeishu}
              onToggle={() => setOpenFeishu((v) => !v)}
            >
              <div className="grid grid-cols-1 gap-3">
                <TextField
                  label="Webhook URL"
                  value={config.feishu.webhookUrl}
                  onChange={(v) => updateNested('feishu', 'webhookUrl', v)}
                  placeholder="https://open.feishu.cn/open-apis/bot/v2/hook/..."
                />
                <TextField
                  label="Secret"
                  type="password"
                  value={config.feishu.secret}
                  onChange={(v) => updateNested('feishu', 'secret', v)}
                  placeholder="签名校验密钥 (可选)"
                />
              </div>
            </NotificationChannel>

            {/* Slack */}
            <NotificationChannel
              id="slack"
              title="Slack"
              icon={<MessageSquare className="w-4 h-4 text-fuchsia-300" />}
              accent="fuchsia"
              open={openSlack}
              onToggle={() => setOpenSlack((v) => !v)}
            >
              <TextField
                label="Webhook URL"
                value={config.slack.webhookUrl}
                onChange={(v) => updateNested('slack', 'webhookUrl', v)}
                placeholder="https://hooks.slack.com/services/..."
              />
            </NotificationChannel>

            {/* Telegram */}
            <NotificationChannel
              id="telegram"
              title="Telegram"
              icon={<Smartphone className="w-4 h-4 text-cyan-300" />}
              accent="cyan"
              open={openTelegram}
              onToggle={() => setOpenTelegram((v) => !v)}
            >
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <TextField
                  label="Bot Token"
                  type="password"
                  value={config.telegram.botToken}
                  onChange={(v) => updateNested('telegram', 'botToken', v)}
                  placeholder="123456:ABC-DEF..."
                />
                <TextField
                  label="Chat ID"
                  value={config.telegram.chatId}
                  onChange={(v) => updateNested('telegram', 'chatId', v)}
                  placeholder="-1001234567890"
                />
              </div>
            </NotificationChannel>
          </div>
        ) : null}
      </div>
      {/* 底部信息 */}
      </div>
      </div>
      </div>
      </div>
      </div>
      </div>
      </div>

      {/* Toast */}
      <Toast toast={toast} onClose={() => setToast(null)} />
    </div>
