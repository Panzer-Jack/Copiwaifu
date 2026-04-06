import {
  AGENT_STATE,
  APP_LANGUAGE,
  WINDOW_SIZE_PRESET,
} from './types/agent'
import type {
  AgentType,
  AppLanguage,
  TAgentState,
  WindowSizePreset,
} from './types/agent'

type LanguageCopy = {
  menu: {
    close: string
    settings: string
    exit: string
  }
  settings: {
    eyebrow: string
    title: string
    description: string
    versionLabel: string
    autoStartLabel: string
    autoStartHint: string
    languageLabel: string
    nameLabel: string
    namePlaceholder: string
    nameCount: (count: number, max: number) => string
    uploadModelLabel: string
    chooseModelDirectoryTitle: string
    validating: string
    chooseDirectory: string
    useDefaultModel: string
    builtInModelPath: string
    switchedToDefaultModel: string
    modelValidated: string
    windowSizeLabel: string
    actionGroupBindingLabel: string
    noBinding: string
    cancel: string
    save: string
    saving: string
    saveSuccess: string
    nameRequired: string
    nameTooLong: (max: number) => string
  }
  status: {
    launchFailed: string
    syncing: string
  }
  stateLabels: Record<TAgentState, string>
  windowSizeLabels: Record<WindowSizePreset, string>
  visibilityLabel: (visible: boolean) => string
  pet: {
    greetings: (name: string) => string[]
    thinking: (name: string) => string
    toolUse: (name: string, toolName: string | null) => string
    error: (name: string) => string
    complete: (name: string) => string
    needsAttention: (name: string) => string
    idleResume: (agentLabel: string, name: string) => string
  }
}

const LANGUAGE_COPY: Record<AppLanguage, LanguageCopy> = {
  [APP_LANGUAGE.ENGLISH]: {
    menu: {
      close: 'Close Menu',
      settings: 'Settings',
      exit: 'Exit',
    },
    settings: {
      eyebrow: 'Copiwaifu',
      title: 'Pet Settings',
      description: 'Manage the name, language, model, size, and motion bindings. Changes apply immediately.',
      versionLabel: 'Version',
      autoStartLabel: 'Auto Start',
      autoStartHint: 'Sync launch at login after saving.',
      languageLabel: 'Language',
      nameLabel: 'Name',
      namePlaceholder: 'Yulia',
      nameCount: (count, max) => `Current ${count}/${max}`,
      uploadModelLabel: 'Upload Model',
      chooseModelDirectoryTitle: 'Choose Live2D Model Folder',
      validating: 'Validating...',
      chooseDirectory: 'Choose Folder',
      useDefaultModel: 'Use Default Model',
      builtInModelPath: 'Using the built-in Hiyori model',
      switchedToDefaultModel: 'Switched back to the built-in default model.',
      modelValidated: 'Model folder validated.',
      windowSizeLabel: 'Window Size',
      actionGroupBindingLabel: 'Action Group Binding',
      noBinding: 'Leave empty to auto-detect',
      cancel: 'Cancel',
      save: 'Save',
      saving: 'Saving...',
      saveSuccess: 'Settings saved and applied.',
      nameRequired: 'Name cannot be empty.',
      nameTooLong: max => `Name can be up to ${max} characters.`,
    },
    status: {
      launchFailed: 'Launch Failed',
      syncing: 'Syncing companion status...',
    },
    stateLabels: {
      [AGENT_STATE.IDLE]: 'Idle',
      [AGENT_STATE.THINKING]: 'Thinking',
      [AGENT_STATE.TOOL_USE]: 'Tool Use',
      [AGENT_STATE.ERROR]: 'Error',
      [AGENT_STATE.COMPLETE]: 'Complete',
      [AGENT_STATE.NEEDS_ATTENTION]: 'Needs Attention',
    },
    windowSizeLabels: {
      [WINDOW_SIZE_PRESET.SMALL]: 'Small',
      [WINDOW_SIZE_PRESET.MEDIUM]: 'Medium',
      [WINDOW_SIZE_PRESET.LARGE]: 'Large',
    },
    visibilityLabel: visible => (visible ? 'Hide' : 'Show'),
    pet: {
      greetings: name => [
        `${name} is keeping an eye on your AI sessions.`,
        `${name} is on standby. Call me when you need me.`,
        `${name} is watching your tools and approval prompts.`,
      ],
      thinking: name => `${name} is thinking...`,
      toolUse: (name, toolName) => toolName
        ? `${name} is running: ${toolName}`
        : `${name} is working...`,
      error: name => `${name} ran into an error.`,
      complete: name => `${name} finished the task!`,
      needsAttention: name => `${name} needs your attention!`,
      idleResume: (agentLabel, name) => `${agentLabel} wrapped up this turn. ${name} is back on it.`,
    },
  },
  [APP_LANGUAGE.CHINESE]: {
    menu: {
      close: '关闭菜单',
      settings: '设置',
      exit: '退出',
    },
    settings: {
      eyebrow: 'Copiwaifu',
      title: '桌宠设置',
      description: '控制名字、语言、模型、尺寸和状态动作组绑定。保存后主窗口会立即同步。',
      versionLabel: '当前版本',
      autoStartLabel: '开机自启',
      autoStartHint: '保存后同步系统开机自启状态。',
      languageLabel: '语言',
      nameLabel: '名字',
      namePlaceholder: 'Yulia',
      nameCount: (count, max) => `当前 ${count}/${max}`,
      uploadModelLabel: '上传模型',
      chooseModelDirectoryTitle: '选择 Live2D 模型目录',
      validating: '校验中...',
      chooseDirectory: '选择目录',
      useDefaultModel: '使用默认模型',
      builtInModelPath: '当前使用内置 Hiyori 模型',
      switchedToDefaultModel: '将切换回内置默认模型。',
      modelValidated: '模型目录校验通过。',
      windowSizeLabel: '窗口尺寸',
      actionGroupBindingLabel: '动作组绑定',
      noBinding: '留空则自动识别',
      cancel: '取消',
      save: '保存',
      saving: '保存中...',
      saveSuccess: '设置已保存并立即生效。',
      nameRequired: '名字不能为空。',
      nameTooLong: max => `名字最多支持 ${max} 个字符。`,
    },
    status: {
      launchFailed: '启动失败',
      syncing: '正在同步桌宠状态...',
    },
    stateLabels: {
      [AGENT_STATE.IDLE]: '空闲',
      [AGENT_STATE.THINKING]: '思考中',
      [AGENT_STATE.TOOL_USE]: '工具调用',
      [AGENT_STATE.ERROR]: '出错',
      [AGENT_STATE.COMPLETE]: '完成',
      [AGENT_STATE.NEEDS_ATTENTION]: '需要关注',
    },
    windowSizeLabels: {
      [WINDOW_SIZE_PRESET.SMALL]: '小',
      [WINDOW_SIZE_PRESET.MEDIUM]: '中',
      [WINDOW_SIZE_PRESET.LARGE]: '大',
    },
    visibilityLabel: visible => (visible ? '隐藏' : '显示'),
    pet: {
      greetings: name => [
        `${name} 在这里继续盯着你的 AI 会话。`,
        `${name} 已待命，有需要随时喊我。`,
        `${name} 会把工具状态和授权请求都看住。`,
      ],
      thinking: name => `${name} 正在思考中...`,
      toolUse: (name, toolName) => toolName
        ? `${name} 正在执行：${toolName}`
        : `${name} 正在执行操作...`,
      error: name => `${name} 捕获到一个错误。`,
      complete: name => `${name} 完成了任务！`,
      needsAttention: name => `${name} 需要你的关注！`,
      idleResume: (agentLabel, name) => `${agentLabel} 这轮处理完成了，${name} 已接住。`,
    },
  },
}

export function getLanguageCopy(language: AppLanguage) {
  return LANGUAGE_COPY[language] ?? LANGUAGE_COPY[APP_LANGUAGE.ENGLISH]
}

export function formatAgentLabel(agent: AgentType | null, language: AppLanguage) {
  if (agent === 'claude-code') {
    return 'Claude Code'
  }
  if (agent === 'copilot') {
    return 'Copilot'
  }
  if (agent === 'codex') {
    return 'Codex'
  }

  return language === APP_LANGUAGE.CHINESE ? 'AI' : 'AI'
}
