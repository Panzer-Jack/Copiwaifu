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
  updater: {
    checkFailedTitle: string
    checkFailedMessage: string
    installFailedTitle: string
    installFailedMessage: string
    manualDownloadMessage: string
    openWebsite: string
    retryLater: string
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
    websiteLabel: string
    actionGroupBindingLabel: string
    noBinding: string
    loadingActionGroups: string
    noActionGroupsFound: string
    actionGroupOptionsLoaded: (count: number) => string
    manualBindingStatus: (group: string) => string
    autoBindingStatus: (group: string) => string
    unresolvedBindingStatus: string
    manualBindingOption: (group: string) => string
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
    thinking: (agentLabel: string, name: string) => string
    toolUse: (agentLabel: string, name: string, toolName: string | null) => string
    error: (agentLabel: string, name: string) => string
    complete: (agentLabel: string, name: string) => string
    needsAttention: (agentLabel: string, name: string) => string
    idleResume: (agentLabel: string, name: string) => string
  }
}

const LANGUAGE_COPY: Record<AppLanguage, LanguageCopy> = {
  [APP_LANGUAGE.ENGLISH]: {
    menu: {
      close: 'Hide Menu',
      settings: 'Desktop Pet Settings',
      exit: 'Exit',
    },
    updater: {
      checkFailedTitle: 'Update Connection Failed',
      checkFailedMessage: 'Copiwaifu could not reach the update server for now. It will quietly try again later.',
      installFailedTitle: 'Update Installation Failed',
      installFailedMessage: 'A new version was detected, but something went wrong while downloading or installing it. Please try again later.',
      manualDownloadMessage: 'You can also visit the official website and manually bring back the latest installer.',
      openWebsite: 'Open Website',
      retryLater: 'Retry Later',
    },
    settings: {
      eyebrow: 'Copiwaifu',
      title: 'Desktop Pet Settings',
      description: 'Adjust your assistant\'s name, language, model form, size, and motion bindings here. Changes sync immediately after saving.',
      versionLabel: 'Current Version',
      autoStartLabel: 'Launch at Login',
      autoStartHint: 'After saving, this will sync with the system login launch setting so she can boot up with you.',
      languageLabel: 'Language',
      nameLabel: 'Character Name',
      namePlaceholder: 'Yulia',
      nameCount: (count, max) => `Name progress ${count}/${max}`,
      uploadModelLabel: 'Change Model',
      chooseModelDirectoryTitle: 'Choose Live2D Model Folder',
      validating: 'Checking model...',
      chooseDirectory: 'Choose Folder',
      useDefaultModel: 'Switch Back to Default Form',
      builtInModelPath: 'Currently using the built-in Yulia form',
      switchedToDefaultModel: 'Switched back to the built-in default form.',
      modelValidated: 'Model import complete. Everything looks good.',
      windowSizeLabel: 'Window Size',
      websiteLabel: 'Official Website',
      actionGroupBindingLabel: 'Action Group Binding',
      noBinding: 'Leave it empty and let the system match it automatically',
      loadingActionGroups: 'Browsing the motion library...',
      noActionGroupsFound: 'No usable motion groups have been detected for this model yet.',
      actionGroupOptionsLoaded: count => `${count} motion groups captured through easy-live2d.`,
      manualBindingStatus: group => `Assigned motion: ${group}`,
      autoBindingStatus: group => `Auto matched: ${group}`,
      unresolvedBindingStatus: 'No corresponding motion group has been matched yet.',
      manualBindingOption: group => `${group} (manually assigned)`,
      cancel: 'Cancel',
      save: 'Save Settings',
      saving: 'Writing settings...',
      saveSuccess: 'Settings saved and now in effect.',
      nameRequired: 'Character name cannot be empty.',
      nameTooLong: max => `Character name can be at most ${max} characters.`,
    },
    status: {
      launchFailed: 'Launch Failed',
      syncing: 'Syncing desktop pet status...',
    },
    stateLabels: {
      [AGENT_STATE.IDLE]: 'On Standby',
      [AGENT_STATE.THINKING]: 'Thinking',
      [AGENT_STATE.TOOL_USE]: 'Casting',
      [AGENT_STATE.ERROR]: 'Something Went Wrong',
      [AGENT_STATE.COMPLETE]: 'Task Complete',
      [AGENT_STATE.NEEDS_ATTENTION]: 'Awaiting Commander Confirmation',
    },
    windowSizeLabels: {
      [WINDOW_SIZE_PRESET.SMALL]: 'Petite',
      [WINDOW_SIZE_PRESET.MEDIUM]: 'Standard',
      [WINDOW_SIZE_PRESET.LARGE]: 'Large',
    },
    visibilityLabel: visible => (visible ? 'Hide for Now' : 'Return to Stage'),
    pet: {
      greetings: name => [
        `Commander! ${name} is on station and ready to watch over your AI sessions today.`,
        `${name} is on standby. Just give the order and I will get to work.`,
        `${name} will keep an eye on tool status and approval requests so nothing runs wild.`,
        'Commander! We have finished syncing with CC, Codex, and Copilot.',
        'Let us change this world together!',
      ],
      thinking: (agentLabel, name) => `[${agentLabel}] ${name}'s thought circuits are spinning at full speed...`,
      toolUse: (agentLabel, name, toolName) => toolName
        ? `[${agentLabel}] ${name} is casting a skill: ${toolName}`
        : `[${agentLabel}] ${name} is processing a spell...`,
      error: (agentLabel, name) => `[${agentLabel}] ${name} detected a bit of unusual turbulence on this side.`,
      complete: (agentLabel, name) => `[${agentLabel}] ${name} has taken the task down cleanly.`,
      needsAttention: (agentLabel, name) => `[${agentLabel}] ${name} needs the commander to take a look.`,
      idleResume: (agentLabel, name) => `${agentLabel} has finished this round. ${name} is returning to standby.`,
    },
  },
  [APP_LANGUAGE.CHINESE]: {
    menu: {
      close: '收起菜单',
      settings: '桌宠设定',
      exit: '退场',
    },
    updater: {
      checkFailedTitle: '更新通讯失败',
      checkFailedMessage: 'Copiwaifu 暂时没连上更新服务器，晚点会再悄悄试一次。',
      installFailedTitle: '更新安装失败',
      installFailedMessage: '新版本已经侦测到了，但下载或安装时出了点小状况，请稍后再试。',
      manualDownloadMessage: '也可以前往官网，手动把最新安装包带回来。',
      openWebsite: '打开官网',
      retryLater: '稍后重试',
    },
    settings: {
      eyebrow: 'Copiwaifu',
      title: '桌宠设定',
      description: '在这里替你的看板娘调整名字、语言、模型形态、尺寸和动作绑定，保存后会立刻同步。',
      versionLabel: '当前版本',
      autoStartLabel: '开机自启',
      autoStartHint: '保存后会和系统登录启动状态同步，让她陪你一起开机报到。',
      languageLabel: '语言',
      nameLabel: '角色名',
      namePlaceholder: 'Yulia',
      nameCount: (count, max) => `名字进度 ${count}/${max}`,
      uploadModelLabel: '更换模型',
      chooseModelDirectoryTitle: '选择 Live2D 模型文件夹',
      validating: '模型检查中...',
      chooseDirectory: '选择文件夹',
      useDefaultModel: '换回默认形态',
      builtInModelPath: '当前使用内置 Yulia 形态',
      switchedToDefaultModel: '已切换回内置默认形态。',
      modelValidated: '模型导入完成，状态一切正常。',
      windowSizeLabel: '窗口尺寸',
      websiteLabel: '官网',
      actionGroupBindingLabel: '动作组绑定',
      noBinding: '留空就交给系统自动配对',
      loadingActionGroups: '正在翻动作库...',
      noActionGroupsFound: '这个模型暂时还没识别到可用动作组。',
      actionGroupOptionsLoaded: count => `已通过 easy-live2d 捕捉到 ${count} 个动作组。`,
      manualBindingStatus: group => `钦定动作：${group}`,
      autoBindingStatus: group => `自动配对：${group}`,
      unresolvedBindingStatus: '还没配对到对应动作组。',
      manualBindingOption: group => `${group}（手动钦定）`,
      cancel: '取消',
      save: '保存设定',
      saving: '设定写入中...',
      saveSuccess: '设定已保存，马上生效。',
      nameRequired: '角色名不能为空。',
      nameTooLong: max => `角色名最多支持 ${max} 个字符。`,
    },
    status: {
      launchFailed: '启动失败',
      syncing: '正在同步桌宠状态...',
    },
    stateLabels: {
      [AGENT_STATE.IDLE]: '待机中',
      [AGENT_STATE.THINKING]: '思考中',
      [AGENT_STATE.TOOL_USE]: '施法中',
      [AGENT_STATE.ERROR]: '出了点状况',
      [AGENT_STATE.COMPLETE]: '任务达成',
      [AGENT_STATE.NEEDS_ATTENTION]: '请长官确认',
    },
    windowSizeLabels: {
      [WINDOW_SIZE_PRESET.SMALL]: '小只',
      [WINDOW_SIZE_PRESET.MEDIUM]: '标准',
      [WINDOW_SIZE_PRESET.LARGE]: '大只',
    },
    visibilityLabel: visible => (visible ? '暂时隐身' : '重新登场'),
    pet: {
      greetings: name => [
        `长官！${name} 已经到岗，今天也由我来守着你的 AI 会话。`,
        `${name} 待机中，长官一声令下我就开工。`,
        `${name} 会帮你盯住工具状态和授权请求，不会让它们乱跑。`,
        '长官！我们已经与CC、Codex、Copilot同步完成。',
        '让我们一起改变这个世界吧！',
      ],
      thinking: (agentLabel, name) => `[${agentLabel}] ${name} 的思考回路正在高速运转...`,
      toolUse: (agentLabel, name, toolName) => toolName
        ? `[${agentLabel}] ${name} 正在释放技能：${toolName}`
        : `[${agentLabel}] ${name} 正在施法处理中...`,
      error: (agentLabel, name) => `[${agentLabel}] ${name} 这边捕捉到一点异常波动。`,
      complete: (agentLabel, name) => `[${agentLabel}] ${name} 已经把任务顺利拿下。`,
      needsAttention: (agentLabel, name) => `[${agentLabel}] ${name} 需要长官看一眼。`,
      idleResume: (agentLabel, name) => `${agentLabel} 这一轮结束啦，接下来由 ${name} 继续待机。`,
    },
  },
  [APP_LANGUAGE.JAPANESE]: {
    menu: {
      close: 'メニューを閉じる',
      settings: 'デスクトップペット設定',
      exit: '終了',
    },
    updater: {
      checkFailedTitle: '更新サーバーに接続できませんでした',
      checkFailedMessage: 'Copiwaifu は現在更新サーバーに接続できません。しばらくしてから再試行します。',
      installFailedTitle: '更新のインストールに失敗しました',
      installFailedMessage: '新しいバージョンは見つかりましたが、ダウンロードまたはインストール中に問題が発生しました。後でもう一度お試しください。',
      manualDownloadMessage: '公式サイトから最新のインストーラーを手動でダウンロードすることもできます。',
      openWebsite: '公式サイトを開く',
      retryLater: '後で再試行',
    },
    settings: {
      eyebrow: 'Copiwaifu',
      title: 'デスクトップペット設定',
      description: 'ここでは看板娘の名前、言語、モデル形態、サイズ、モーションバインドを調整できます。保存後すぐに反映されます。',
      versionLabel: '現在のバージョン',
      autoStartLabel: 'ログイン時に起動',
      autoStartHint: '保存後、システムのログイン起動設定と同期し、起動時に一緒に立ち上がります。',
      languageLabel: '言語',
      nameLabel: 'キャラクター名',
      namePlaceholder: 'Yulia',
      nameCount: (count, max) => `名前の長さ ${count}/${max}`,
      uploadModelLabel: 'モデルを変更',
      chooseModelDirectoryTitle: 'Live2D モデルフォルダーを選択',
      validating: 'モデルを確認中...',
      chooseDirectory: 'フォルダーを選択',
      useDefaultModel: 'デフォルト形態に戻す',
      builtInModelPath: '現在は内蔵の Yulia 形態を使用中',
      switchedToDefaultModel: '内蔵のデフォルト形態に戻しました。',
      modelValidated: 'モデルの読み込みが完了し、問題なく利用できます。',
      windowSizeLabel: 'ウィンドウサイズ',
      websiteLabel: '公式サイト',
      actionGroupBindingLabel: 'モーショングループの割り当て',
      noBinding: '空のままにするとシステムが自動で対応づけます',
      loadingActionGroups: 'モーションライブラリを読み込み中...',
      noActionGroupsFound: 'このモデルではまだ利用可能なモーショングループが見つかっていません。',
      actionGroupOptionsLoaded: count => `easy-live2d から ${count} 個のモーショングループを取得しました。`,
      manualBindingStatus: group => `手動指定: ${group}`,
      autoBindingStatus: group => `自動一致: ${group}`,
      unresolvedBindingStatus: '対応するモーショングループはまだ見つかっていません。',
      manualBindingOption: group => `${group}（手動指定）`,
      cancel: 'キャンセル',
      save: '設定を保存',
      saving: '設定を書き込み中...',
      saveSuccess: '設定を保存しました。すぐに反映されます。',
      nameRequired: 'キャラクター名は空にできません。',
      nameTooLong: max => `キャラクター名は最大 ${max} 文字です。`,
    },
    status: {
      launchFailed: '起動に失敗しました',
      syncing: 'デスクトップペットの状態を同期しています...',
    },
    stateLabels: {
      [AGENT_STATE.IDLE]: '待機中',
      [AGENT_STATE.THINKING]: '思考中',
      [AGENT_STATE.TOOL_USE]: 'スキル使用中',
      [AGENT_STATE.ERROR]: '問題が発生しました',
      [AGENT_STATE.COMPLETE]: '完了',
      [AGENT_STATE.NEEDS_ATTENTION]: '確認待ち',
    },
    windowSizeLabels: {
      [WINDOW_SIZE_PRESET.SMALL]: '小さめ',
      [WINDOW_SIZE_PRESET.MEDIUM]: '標準',
      [WINDOW_SIZE_PRESET.LARGE]: '大きめ',
    },
    visibilityLabel: visible => (visible ? 'いったん隠す' : '再登場'),
    pet: {
      greetings: name => [
        `指揮官！${name} は配置につきました。今日も AI セッションを見守ります。`,
        `${name} は待機中です。命令があればすぐに動きます。`,
        `${name} がツールの状態と承認リクエストを見張って、暴走しないようにします。`,
        '指揮官！CC、Codex、Copilot との同期が完了しました。',
        '一緒にこの世界を変えていきましょう！',
      ],
      thinking: (agentLabel, name) => `[${agentLabel}] ${name} の思考回路がフル回転しています...`,
      toolUse: (agentLabel, name, toolName) => toolName
        ? `[${agentLabel}] ${name} がスキルを発動中: ${toolName}`
        : `[${agentLabel}] ${name} が処理を実行しています...`,
      error: (agentLabel, name) => `[${agentLabel}] ${name} が少し異常な揺らぎを検知しました。`,
      complete: (agentLabel, name) => `[${agentLabel}] ${name} がタスクをきれいに片づけました。`,
      needsAttention: (agentLabel, name) => `[${agentLabel}] ${name} が指揮官の確認を求めています。`,
      idleResume: (agentLabel, name) => `${agentLabel} の今回の処理は完了です。${name} は待機に戻ります。`,
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
  if (agent === 'gemini') {
    return 'Gemini'
  }
  if (agent === 'opencode') {
    return 'OpenCode'
  }

  return language === APP_LANGUAGE.CHINESE ? 'AI' : 'AI'
}
