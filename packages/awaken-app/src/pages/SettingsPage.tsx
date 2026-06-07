import { useState, useEffect, useCallback } from 'react';
import { Save, RotateCcw, ChevronDown, CheckCircle2, AlertCircle, Settings2, Mail, MessageSquare, Bot, Globe, Send, Smartphone, Terminal, RefreshCw } from 'lucide-react';

/* ===== Types ===== */
interface NotificationChannelConfig {
  smtpHost: string; smtpPort: number; username: string; password: string; fromAddress: string;
}
interface DingtalkConfig { webhookUrl: string; secret: string; }
interface SimpleWebhook { webhookUrl: string; }
interface FeishuConfig { webhookUrl: string; secret: string; }
interface TelegramConfig { botToken: string; chatId: string; }
interface OpenCodeConfig { zenApiKey: string; autoDiscover: boolean; enableCli: boolean; cliTimeout: number; }
interface MediaGenConfig { openaiApiKey: string; agnesApiKey: string; defaultImageProvider: string; defaultImageModel: string; defaultVideoProvider: string; defaultVideoModel: string; imageSize: string; }
interface XfyunConfig { apiKey: string; region: string; model: string; temperature: number; maxTokens: number; stream: boolean; embeddingModel: string; rerankModel: string; appId: string; ttiModel: string; ttiWidth: number; ttiHeight: number; ttiSteps: number; }

interface AgentConfig {
  agentId: string; model: string; provider: string; temperature: number; maxTokens: number; systemPrompt: string;
  visionProvider: string; visionModel: string; visionApiKey: string; visionBaseUrl: string; maxImageSize: number;
  opencode: OpenCodeConfig; mediaGen: MediaGenConfig; xfyun: XfyunConfig;
  email: NotificationChannelConfig; dingtalk: DingtalkConfig; wecom: SimpleWebhook; feishu: FeishuConfig; slack: SimpleWebhook; telegram: TelegramConfig;
}
interface ToastState { message: string; type: 'success' | 'error'; }

/* ===== Constants ===== */
const MODELS = ['gpt-4o', 'gpt-4o-mini', 'claude-3-5-sonnet', 'claude-3-haiku', 'llama3', 'llama3-70b', 'mistral', 'mixtral', 'qwen3.5-2b', 'agnes-1.5-flash', 'agnes-2.0-flash'];
const PROVIDERS = ['openai', 'anthropic', 'ollama', 'xfyun', 'agnes'];
const VISION_PROVIDERS = ['openai', 'anthropic', 'ollama', 'none'];
const XFYUN_REGIONS = ['华北-北京', '华东-上海', '华南-广州'];
const IMAGE_SIZES = ['1024x1024', '1792x1024', '1024x1792', '512x512', '768x768', '1024x768', '768x1024'];
const IMAGE_MODELS = ['dall-e-3', 'agnes-image-2.0-flash', 'agnes-image-2.1-flash'];
const VIDEO_MODELS = ['agnes-video-v2.0'];
const FREE_MODELS = [
  { id: 'deepseek-v4-flash-free', name: 'DeepSeek V4 Flash Free' },
  { id: 'big-pickle-free', name: 'Big Pickle Free' },
  { id: 'mimo-v2.5-free', name: 'MiMo V2.5 Free' },
  { id: 'nemotron-3-ultra-free', name: 'Nemotron 3 Ultra Free' },
];

const DEFAULT_CONFIG: AgentConfig = {
  agentId: 'assistant', model: 'gpt-4o', provider: 'openai', temperature: 0.7, maxTokens: 2048,
  systemPrompt: 'You are a helpful AI assistant.',
  visionProvider: 'openai', visionModel: 'gpt-4o', visionApiKey: '', visionBaseUrl: '', maxImageSize: 20,
  opencode: { zenApiKey: '', autoDiscover: true, enableCli: true, cliTimeout: 300 },
  mediaGen: { openaiApiKey: '', agnesApiKey: '', defaultImageProvider: 'openai', defaultImageModel: 'dall-e-3', defaultVideoProvider: 'agnes', defaultVideoModel: 'agnes-video-v2.0', imageSize: '1024x1024' },
  xfyun: { apiKey: '', region: '华北-北京', model: 'qwen3.5-2b', temperature: 0.5, maxTokens: 4096, stream: false, embeddingModel: 'sde0a5839', rerankModel: 's125c8e0e', appId: '', ttiModel: '', ttiWidth: 768, ttiHeight: 768, ttiSteps: 20 },
  email: { smtpHost: '', smtpPort: 587, username: '', password: '', fromAddress: '' },
  dingtalk: { webhookUrl: '', secret: '' },
  wecom: { webhookUrl: '' },
  feishu: { webhookUrl: '', secret: '' },
  slack: { webhookUrl: '' },
  telegram: { botToken: '', chatId: '' },
};

/* ===== Sub-components ===== */
function TextField({ label, value, onChange, type = 'text', placeholder = '', readOnly = false, rows }: {
  label: string; value: string; onChange: (v: string) => void; type?: string; placeholder?: string; readOnly?: boolean; rows?: number;
}) {
  return (
    <div>
      <label className="block text-xs font-medium text-gray-400 mb-1.5">{label}</label>
      {rows ? (
        <textarea value={value} onChange={(e) => onChange(e.target.value)} rows={rows} readOnly={readOnly} placeholder={placeholder}
          className="w-full px-3 py-2 rounded-xl text-sm bg-gray-800/40 border border-gray-700/50 text-gray-100 placeholder-gray-600 outline-none focus:border-indigo-500/60 transition-colors resize-none" />
      ) : (
        <input type={type} value={value} onChange={(e) => onChange(e.target.value)} readOnly={readOnly} placeholder={placeholder}
          className="w-full h-10 px-3 rounded-xl text-sm bg-gray-800/40 border border-gray-700/50 text-gray-100 placeholder-gray-600 outline-none focus:border-indigo-500/60 transition-colors" />
      )}
    </div>
  );
}

function SelectField({ label, value, options, onChange }: { label: string; value: string; options: string[]; onChange: (v: string) => void }) {
  return (
    <div>
      <label className="block text-xs font-medium text-gray-400 mb-1.5">{label}</label>
      <select value={value} onChange={(e) => onChange(e.target.value)}
        className="w-full h-10 px-3 rounded-xl text-sm bg-gray-800/40 border border-gray-700/50 text-gray-100 outline-none focus:border-indigo-500/60 transition-colors cursor-pointer">
        {options.map((o) => <option key={o} value={o}>{o}</option>)}
      </select>
    </div>
  );
}

function SliderField({ label, value, min = 0, max = 1, step = 0.1, onChange }: { label: string; value: number; min?: number; max?: number; step?: number; onChange: (v: number) => void }) {
  return (
    <div>
      <div className="flex items-center justify-between mb-1.5">
        <label className="text-xs font-medium text-gray-400">{label}</label>
        <span className="text-xs text-gray-500 font-mono">{value}</span>
      </div>
      <input type="range" min={min} max={max} step={step} value={value} onChange={(e) => onChange(parseFloat(e.target.value))}
        className="w-full h-1.5 rounded-full appearance-none cursor-pointer bg-gray-700 accent-indigo-500" />
    </div>
  );
}

function Toast({ toast, onClose }: { toast: ToastState; onClose: () => void }) {
  useEffect(() => { const t = setTimeout(onClose, 3500); return () => clearTimeout(t); }, [onClose]);
  const isSuccess = toast.type === 'success';
  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 animate-slide-up">
      <div className={`flex items-center gap-2.5 px-5 py-3 rounded-2xl shadow-2xl border text-sm font-medium backdrop-blur-xl ${isSuccess ? 'bg-green-900/60 border-green-700/40 text-green-200' : 'bg-red-900/60 border-red-700/40 text-red-200'}`}>
        {isSuccess ? <CheckCircle2 className="w-4 h-4" /> : <AlertCircle className="w-4 h-4" />}
        <span>{toast.message}</span>
        <button onClick={onClose} className="ml-2 opacity-60 hover:opacity-100">✕</button>
      </div>
    </div>
  );
}

/* ===== Notification Channel ===== */
function NotificationChannel({ icon, label, color, enabled, onToggle, children, expanded, onExpand }: {
  icon: React.ReactNode; label: string; color: string; enabled: boolean; onToggle: (v: boolean) => void;
  children: React.ReactNode; expanded: boolean; onExpand: (v: boolean) => void;
}) {
  return (
    <div className="rounded-xl border border-gray-800/40 overflow-hidden">
      <button type="button" onClick={() => onExpand(!expanded)} className="w-full flex items-center gap-3 px-4 py-3 bg-gray-800/20 hover:bg-gray-800/40 transition-colors">
        <span className={`text-${color}`}>{icon}</span>
        <span className="flex-1 text-left text-sm font-medium text-gray-200">{label}</span>
        <label onClick={(e) => e.stopPropagation()} className="relative inline-flex items-center cursor-pointer">
          <input type="checkbox" checked={enabled} onChange={(e) => onToggle(e.target.checked)} className="sr-only peer" />
          <div className="w-8 h-4.5 rounded-full bg-gray-700 peer-checked:bg-emerald-500/60 peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3.5 after:w-3.5 after:transition-all" />
        </label>
        <ChevronDown className={`w-4 h-4 text-gray-500 transition-transform duration-200 ${expanded ? 'rotate-180' : ''}`} />
      </button>
      {expanded && <div className="px-4 py-4 space-y-3 border-t border-gray-800/30 animate-fade-in">{children}</div>}
    </div>
  );
}

/* ===== Main component ===== */
export function SettingsPage() {
  const [config, setConfig] = useState<AgentConfig>(DEFAULT_CONFIG);
  const [loading, setLoading] = useState(true);
  const [dirty, setDirty] = useState(false);
  const [visionExpanded, setVisionExpanded] = useState(false);
  const [openCodeExpanded, setOpenCodeExpanded] = useState(false);
  const [mediaGenExpanded, setMediaGenExpanded] = useState(false);
  const [xfyunExpanded, setXfyunExpanded] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [notifExpanded, setNotifExpanded] = useState<Record<string, boolean>>({});

  useEffect(() => {
    try {
      const saved = localStorage.getItem('agent_config');
      if (saved) setConfig((prev) => ({ ...prev, ...JSON.parse(saved) }));
    } catch { /* ignore */ }
    setLoading(false);
  }, []);

  const update = useCallback(<K extends keyof AgentConfig>(key: K, value: AgentConfig[K]) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
    setDirty(true);
  }, []);

  const updateNested = useCallback((section: string, field: string, value: unknown) => {
    setConfig((prev) => {
      const s = (prev as Record<string, unknown>)[section] as Record<string, unknown>;
      return { ...prev, [section]: { ...s, [field]: value } };
    });
    setDirty(true);
  }, []);

  const handleSave = useCallback(() => {
    localStorage.setItem('agent_config', JSON.stringify(config));
    setDirty(false);
    setToast({ message: '配置已保存', type: 'success' });
  }, [config]);

  const handleReset = useCallback(() => {
    setConfig(DEFAULT_CONFIG);
    setDirty(true);
    setToast({ message: '已重置为默认值', type: 'success' });
  }, []);

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="flex gap-1">
          <span className="w-2 h-2 rounded-full bg-indigo-400/40 animate-bounce" style={{ animationDelay: '0ms' }} />
          <span className="w-2 h-2 rounded-full bg-indigo-400/40 animate-bounce" style={{ animationDelay: '200ms' }} />
          <span className="w-2 h-2 rounded-full bg-indigo-400/40 animate-bounce" style={{ animationDelay: '400ms' }} />
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-3xl mx-auto px-6 py-8 space-y-8">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-400/20 to-purple-500/10 border border-indigo-400/20 flex items-center justify-center">
              <Settings2 className="w-5 h-5 text-indigo-400" />
            </div>
            <div>
              <h1 className="text-lg font-semibold text-gray-100">Agent 配置</h1>
              <p className="text-xs text-gray-500 mt-0.5">管理 AI 助手的模型与行为参数</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button onClick={handleReset} className="flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-xs font-medium text-gray-400 bg-gray-800/30 border border-gray-700/30 hover:text-gray-200 hover:bg-gray-700/40 transition-all duration-200">
              <RotateCcw className="w-3.5 h-3.5" />重置
            </button>
            <button onClick={handleSave} disabled={!dirty}
              className="flex items-center gap-1.5 px-4 py-2 rounded-xl text-xs font-medium bg-gradient-to-r from-indigo-500 to-indigo-400 text-white shadow-lg shadow-indigo-500/20 hover:from-indigo-600 hover:to-indigo-500 active:scale-[0.98] disabled:opacity-40 disabled:cursor-not-allowed transition-all duration-200">
              <Save className="w-3.5 h-3.5" />保存
            </button>
          </div>
        </div>

        {/* Basic Info */}
        <div className="p-5 rounded-2xl bg-gray-900/40 border border-gray-800/40 backdrop-blur-sm space-y-5">
          <div className="flex items-center gap-2 pb-2 border-b border-gray-800/30">
            <div className="w-1.5 h-1.5 rounded-full bg-indigo-400" />
            <h2 className="text-sm font-semibold text-gray-200">基本信息</h2>
          </div>
          <TextField label="Agent ID" value={config.agentId} onChange={() => {}} readOnly />
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <SelectField label="模型" value={config.model} options={MODELS} onChange={(v) => update('model', v)} />
            <SelectField label="Provider" value={config.provider} options={PROVIDERS} onChange={(v) => update('provider', v)} />
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <SliderField label="Temperature" value={config.temperature} min={0} max={2} step={0.1} onChange={(v) => update('temperature', v)} />
            <TextField label="Max Tokens" value={String(config.maxTokens)} onChange={(v) => update('maxTokens', Math.max(1, Math.min(128000, Number(v) || 2048)))} type="number" placeholder="2048" />
          </div>
          <TextField label="System Prompt" value={config.systemPrompt} onChange={(v) => update('systemPrompt', v)} rows={3} placeholder="You are a helpful AI assistant." />
        </div>

        {/* Vision */}
        <div className="p-5 rounded-2xl bg-gray-900/40 border border-gray-800/40 backdrop-blur-sm space-y-5">
          <button type="button" onClick={() => setVisionExpanded((v) => !v)} className="w-full flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-indigo-400" />
              <h2 className="text-sm font-semibold text-gray-200">Vision 配置</h2>
              <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${config.visionProvider === 'none' ? 'bg-gray-800/40 text-gray-500' : 'bg-indigo-400/10 text-indigo-400 border border-indigo-400/20'}`}>
                {config.visionProvider === 'none' ? '已禁用' : config.visionProvider}
              </span>
            </div>
            <ChevronDown className={`w-4 h-4 text-gray-400 transition-transform duration-200 ${visionExpanded ? 'rotate-180' : ''}`} />
          </button>
          {visionExpanded && (
            <div className="space-y-4 animate-fade-in">
              <SelectField label="Vision Provider" value={config.visionProvider} options={VISION_PROVIDERS} onChange={(v) => update('visionProvider', v)} />
              {config.visionProvider !== 'none' && (
                <>
                  <TextField label="Vision Model" value={config.visionModel} onChange={(v) => update('visionModel', v)} placeholder="gpt-4o" />
                  <TextField label="API Key" value={config.visionApiKey} onChange={(v) => update('visionApiKey', v)} type="password" placeholder="sk-..." />
                  {config.visionProvider === 'ollama' && (
                    <TextField label="Base URL" value={config.visionBaseUrl} onChange={(v) => update('visionBaseUrl', v)} placeholder="http://localhost:11434" />
                  )}
                  <SliderField label="Max Image Size (MB)" value={config.maxImageSize} min={1} max={50} step={1} onChange={(v) => update('maxImageSize', v)} />
                </>
              )}
            </div>
          )}
        </div>

        {/* OpenCode */}
        <div className="p-5 rounded-2xl bg-gray-900/40 border border-gray-800/40 backdrop-blur-sm space-y-5">
          <button type="button" onClick={() => setOpenCodeExpanded((v) => !v)} className="w-full flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-cyan-400" />
              <h2 className="text-sm font-semibold text-gray-200">OpenCode</h2>
              <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${config.opencode.enableCli ? 'bg-cyan-400/10 text-cyan-400 border border-cyan-400/20' : 'bg-gray-800/40 text-gray-500'}`}>
                {config.opencode.enableCli ? 'CLI 已启用' : 'CLI 已禁用'}
              </span>
            </div>
            <ChevronDown className={`w-4 h-4 text-gray-400 transition-transform duration-200 ${openCodeExpanded ? 'rotate-180' : ''}`} />
          </button>
          {openCodeExpanded && (
            <div className="space-y-4 animate-fade-in">
              <p className="text-xs text-gray-500">OpenCode 是一个开源 AI 编程代理。自动发现并免费使用来自 OpenCode Zen 的免费模型。</p>
              <TextField label="Zen API Key" value={config.opencode.zenApiKey} onChange={(v) => updateNested('opencode', 'zenApiKey', v)} type="password" placeholder="可选 — 免费模型无需 Key" />
              <div className="flex items-center justify-between py-2">
                <div>
                  <span className="text-xs font-medium text-gray-300">自动发现免费模型</span>
                  <p className="text-[11px] text-gray-500 mt-0.5">自动从 OpenCode Zen 发现可用免费模型</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" checked={config.opencode.autoDiscover} onChange={(e) => updateNested('opencode', 'autoDiscover', e.target.checked)} className="sr-only peer" />
                  <div className="w-8 h-4.5 rounded-full bg-gray-700 peer-checked:bg-cyan-500/60 peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3.5 after:w-3.5 after:transition-all" />
                </label>
              </div>
              <div className="flex items-center justify-between py-2">
                <div>
                  <span className="text-xs font-medium text-gray-300">启用 CLI 工具</span>
                  <p className="text-[11px] text-gray-500 mt-0.5">允许 Agent 调用 OpenCode CLI 进行代码生成</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" checked={config.opencode.enableCli} onChange={(e) => updateNested('opencode', 'enableCli', e.target.checked)} className="sr-only peer" />
                  <div className="w-8 h-4.5 rounded-full bg-gray-700 peer-checked:bg-cyan-500/60 peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3.5 after:w-3.5 after:transition-all" />
                </label>
              </div>
              <TextField label="CLI 超时 (秒)" value={String(config.opencode.cliTimeout)} onChange={(v) => updateNested('opencode', 'cliTimeout', Math.max(30, Math.min(3600, Number(v) || 300)))} type="number" placeholder="300" />
              <div className="rounded-xl bg-gray-800/30 border border-gray-700/30 p-4 space-y-2">
                <div className="flex items-center gap-2 text-xs text-gray-400">
                  <Terminal className="w-3.5 h-3.5" /><span className="font-medium text-gray-300">免费模型列表</span>
                </div>
                {FREE_MODELS.map((m) => (
                  <div key={m.id} className="flex items-center gap-2 text-xs text-gray-500">
                    <RefreshCw className="w-3 h-3 text-cyan-400/60" /><span className="text-gray-300 font-mono">{m.name}</span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-green-900/30 text-green-400 border border-green-700/30">免费</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* MediaGen */}
        <div className="p-5 rounded-2xl bg-gray-900/40 border border-gray-800/40 backdrop-blur-sm space-y-5">
          <button type="button" onClick={() => setMediaGenExpanded((v) => !v)} className="w-full flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-pink-400" />
              <h2 className="text-sm font-semibold text-gray-200">媒体生成</h2>
              <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${config.mediaGen.openaiApiKey || config.mediaGen.agnesApiKey ? 'bg-pink-400/10 text-pink-400 border border-pink-400/20' : 'bg-gray-800/40 text-gray-500'}`}>
                {config.mediaGen.openaiApiKey || config.mediaGen.agnesApiKey ? '已配置' : '未配置'}
              </span>
            </div>
            <ChevronDown className={`w-4 h-4 text-gray-400 transition-transform duration-200 ${mediaGenExpanded ? 'rotate-180' : ''}`} />
          </button>
          {mediaGenExpanded && (
            <div className="space-y-4 animate-fade-in">
              <p className="text-xs text-gray-500">通过 AI 生成图片和视频。支持 OpenAI DALL-E 3、Agnes AI Image 和 Video。</p>
              <TextField label="OpenAI API Key" value={config.mediaGen.openaiApiKey} onChange={(v) => updateNested('mediaGen', 'openaiApiKey', v)} type="password" placeholder="sk-...（DALL-E 3 图片生成）" />
              <TextField label="Agnes AI API Key" value={config.mediaGen.agnesApiKey} onChange={(v) => updateNested('mediaGen', 'agnesApiKey', v)} type="password" placeholder="（Agnes 图片/视频生成）" />
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <SelectField label="默认图片供应商" value={config.mediaGen.defaultImageProvider} options={['openai', 'agnes']} onChange={(v) => updateNested('mediaGen', 'defaultImageProvider', v)} />
                <SelectField label="默认图片模型" value={config.mediaGen.defaultImageModel} options={IMAGE_MODELS} onChange={(v) => updateNested('mediaGen', 'defaultImageModel', v)} />
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <SelectField label="默认视频供应商" value={config.mediaGen.defaultVideoProvider} options={['agnes']} onChange={(v) => updateNested('mediaGen', 'defaultVideoProvider', v)} />
                <SelectField label="默认视频模型" value={config.mediaGen.defaultVideoModel} options={VIDEO_MODELS} onChange={(v) => updateNested('mediaGen', 'defaultVideoModel', v)} />
              </div>
              <SelectField label="图片尺寸" value={config.mediaGen.imageSize} options={IMAGE_SIZES} onChange={(v) => updateNested('mediaGen', 'imageSize', v)} />
            </div>
          )}
        </div>

        {/* Xfyun */}
        <div className="p-5 rounded-2xl bg-gray-900/40 border border-gray-800/40 backdrop-blur-sm space-y-5">
          <button type="button" onClick={() => setXfyunExpanded((v) => !v)} className="w-full flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-amber-400" />
              <h2 className="text-sm font-semibold text-gray-200">讯飞星辰 MaaS</h2>
              <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${config.xfyun.apiKey ? 'bg-amber-400/10 text-amber-400 border border-amber-400/20' : 'bg-gray-800/40 text-gray-500'}`}>
                {config.xfyun.apiKey ? '已配置' : '未配置'}
              </span>
            </div>
            <ChevronDown className={`w-4 h-4 text-gray-400 transition-transform duration-200 ${xfyunExpanded ? 'rotate-180' : ''}`} />
          </button>
          {xfyunExpanded && (
            <div className="space-y-4 animate-fade-in">
              <p className="text-xs text-gray-500">讯飞星辰 MaaS 平台提供星火大模型推理服务与 Embedding & Rerank 服务，使用 OpenAI 兼容 API 协议。</p>
              <TextField label="API Key" value={config.xfyun.apiKey} onChange={(v) => updateNested('xfyun', 'apiKey', v)} type="password" placeholder="从讯飞开放平台获取" />
              <SelectField label="接入区域" value={config.xfyun.region} options={XFYUN_REGIONS} onChange={(v) => updateNested('xfyun', 'region', v)} />
              <SelectField label="模型 (对话)" value={config.xfyun.model} options={['qwen3.5-2b']} onChange={(v) => updateNested('xfyun', 'model', v)} />
              <div className="border-t border-gray-800/20 pt-4 mt-2">
                <p className="text-xs font-medium text-gray-400 mb-3">Embedding & Rerank 服务</p>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <TextField label="Embedding 模型" value={config.xfyun.embeddingModel} onChange={(v) => updateNested('xfyun', 'embeddingModel', v)} placeholder="sde0a5839" />
                  <TextField label="Rerank 模型" value={config.xfyun.rerankModel} onChange={(v) => updateNested('xfyun', 'rerankModel', v)} placeholder="s125c8e0e" />
                </div>
              </div>
              <div className="border-t border-gray-800/20 pt-4 mt-2">
                <p className="text-xs font-medium text-gray-400 mb-3">TTI 图片生成</p>
                <TextField label="App ID" value={config.xfyun.appId} onChange={(v) => updateNested('xfyun', 'appId', v)} placeholder="从开放平台控制台获取" />
                <TextField label="TTI 模型" value={config.xfyun.ttiModel} onChange={(v) => updateNested('xfyun', 'ttiModel', v)} placeholder="模型 domain ID" />
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <TextField label="图片宽度" value={String(config.xfyun.ttiWidth)} onChange={(v) => updateNested('xfyun', 'ttiWidth', Number(v) || 768)} type="number" placeholder="768" />
                  <TextField label="图片高度" value={String(config.xfyun.ttiHeight)} onChange={(v) => updateNested('xfyun', 'ttiHeight', Number(v) || 768)} type="number" placeholder="768" />
                </div>
                <TextField label="生成步数" value={String(config.xfyun.ttiSteps)} onChange={(v) => updateNested('xfyun', 'ttiSteps', Math.max(1, Math.min(50, Number(v) || 20)))} type="number" placeholder="20" />
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <SliderField label="Temperature" value={config.xfyun.temperature} min={0} max={1} step={0.1} onChange={(v) => updateNested('xfyun', 'temperature', v)} />
                <TextField label="Max Tokens" value={String(config.xfyun.maxTokens)} onChange={(v) => updateNested('xfyun', 'maxTokens', Math.max(1, Math.min(16384, Number(v) || 4096)))} type="number" placeholder="4096" />
              </div>
              <div className="flex items-center justify-between py-2">
                <div>
                  <span className="text-xs font-medium text-gray-300">流式输出</span>
                  <p className="text-[11px] text-gray-500 mt-0.5">启用 SSE 流式响应</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" checked={config.xfyun.stream} onChange={(e) => updateNested('xfyun', 'stream', e.target.checked)} className="sr-only peer" />
                  <div className="w-8 h-4.5 rounded-full bg-gray-700 peer-checked:bg-amber-500/60 peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3.5 after:w-3.5 after:transition-all" />
                </label>
              </div>
              <div className="rounded-xl bg-gray-800/30 border border-gray-700/30 p-4">
                <p className="text-[11px] text-gray-500">支持区域：华北-北京 | 华东-上海 | 华南-广州 · 兼容 OpenAI 格式</p>
              </div>
            </div>
          )}
        </div>

        {/* Notifications */}
        <div className="p-5 rounded-2xl bg-gray-900/40 border border-gray-800/40 backdrop-blur-sm space-y-5">
          <div className="flex items-center gap-2 pb-2 border-b border-gray-800/30">
            <div className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
            <h2 className="text-sm font-semibold text-gray-200">通知通道</h2>
            <span className="text-[10px] text-gray-500 ml-auto">配置完成后 Agent 可发送消息到各平台</span>
          </div>

          <NotificationChannel icon={<Mail className="w-4 h-4" />} label="Email (SMTP)" color="text-amber-400" enabled={!!config.email.smtpHost}
            onToggle={(on) => updateNested('email', 'smtpHost', on ? 'smtp.example.com' : '')}
            expanded={!!notifExpanded.email} onExpand={(v) => setNotifExpanded({ ...notifExpanded, email: v })}>
            <TextField label="SMTP Host" value={config.email.smtpHost} onChange={(v) => updateNested('email', 'smtpHost', v)} placeholder="smtp.gmail.com" />
            <TextField label="SMTP Port" value={String(config.email.smtpPort)} onChange={(v) => updateNested('email', 'smtpPort', Number(v) || 587)} type="number" placeholder="587" />
            <TextField label="Username" value={config.email.username} onChange={(v) => updateNested('email', 'username', v)} placeholder="user@gmail.com" />
            <TextField label="Password" value={config.email.password} onChange={(v) => updateNested('email', 'password', v)} type="password" placeholder="app password" />
            <TextField label="From Address" value={config.email.fromAddress} onChange={(v) => updateNested('email', 'fromAddress', v)} placeholder="noreply@example.com" />
          </NotificationChannel>

          <NotificationChannel icon={<MessageSquare className="w-4 h-4" />} label="钉钉 (DingTalk)" color="text-blue-400" enabled={!!config.dingtalk.webhookUrl}
            onToggle={(on) => updateNested('dingtalk', 'webhookUrl', on ? 'https://oapi.dingtalk.com/robot/send?access_token=' : '')}
            expanded={!!notifExpanded.dingtalk} onExpand={(v) => setNotifExpanded({ ...notifExpanded, dingtalk: v })}>
            <TextField label="Webhook URL" value={config.dingtalk.webhookUrl} onChange={(v) => updateNested('dingtalk', 'webhookUrl', v)} placeholder="https://oapi.dingtalk.com/robot/send?access_token=..." />
            <TextField label="Secret" value={config.dingtalk.secret} onChange={(v) => updateNested('dingtalk', 'secret', v)} type="password" placeholder="可选" />
          </NotificationChannel>

          <NotificationChannel icon={<Bot className="w-4 h-4" />} label="企业微信 (WeCom)" color="text-green-400" enabled={!!config.wecom.webhookUrl}
            onToggle={(on) => updateNested('wecom', 'webhookUrl', on ? 'https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=' : '')}
            expanded={!!notifExpanded.wecom} onExpand={(v) => setNotifExpanded({ ...notifExpanded, wecom: v })}>
            <TextField label="Webhook URL" value={config.wecom.webhookUrl} onChange={(v) => updateNested('wecom', 'webhookUrl', v)} placeholder="https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=..." />
          </NotificationChannel>

          <NotificationChannel icon={<Send className="w-4 h-4" />} label="飞书 (Feishu)" color="text-red-400" enabled={!!config.feishu.webhookUrl}
            onToggle={(on) => updateNested('feishu', 'webhookUrl', on ? 'https://open.feishu.cn/open-apis/bot/v2/hook/' : '')}
            expanded={!!notifExpanded.feishu} onExpand={(v) => setNotifExpanded({ ...notifExpanded, feishu: v })}>
            <TextField label="Webhook URL" value={config.feishu.webhookUrl} onChange={(v) => updateNested('feishu', 'webhookUrl', v)} placeholder="https://open.feishu.cn/open-apis/bot/v2/hook/..." />
            <TextField label="Secret" value={config.feishu.secret} onChange={(v) => updateNested('feishu', 'secret', v)} type="password" placeholder="可选" />
          </NotificationChannel>

          <NotificationChannel icon={<Globe className="w-4 h-4" />} label="Slack" color="text-purple-400" enabled={!!config.slack.webhookUrl}
            onToggle={(on) => updateNested('slack', 'webhookUrl', on ? 'https://hooks.slack.com/services/' : '')}
            expanded={!!notifExpanded.slack} onExpand={(v) => setNotifExpanded({ ...notifExpanded, slack: v })}>
            <TextField label="Webhook URL" value={config.slack.webhookUrl} onChange={(v) => updateNested('slack', 'webhookUrl', v)} placeholder="https://hooks.slack.com/services/T00/B00/..." />
          </NotificationChannel>

          <NotificationChannel icon={<Smartphone className="w-4 h-4" />} label="Telegram" color="text-sky-400" enabled={!!config.telegram.botToken}
            onToggle={(on) => updateNested('telegram', 'botToken', on ? 'bot' : '')}
            expanded={!!notifExpanded.telegram} onExpand={(v) => setNotifExpanded({ ...notifExpanded, telegram: v })}>
            <TextField label="Bot Token" value={config.telegram.botToken} onChange={(v) => updateNested('telegram', 'botToken', v)} type="password" placeholder="123456:ABC-DEF..." />
            <TextField label="Chat ID" value={config.telegram.chatId} onChange={(v) => updateNested('telegram', 'chatId', v)} placeholder="-1001234567890" />
          </NotificationChannel>
        </div>

        <p className="text-[10px] text-gray-600 text-center pb-4">修改会自动保存在本地</p>
      </div>
      {toast && <Toast toast={toast} onClose={() => setToast(null)} />}
    </div>
  );
}
