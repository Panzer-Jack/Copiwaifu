<script setup lang="ts">
import type {
  AppBootstrap,
  AppSettings,
  AiTalkProviderProfile,
  ImportedModelResult,
  ModelScanResult,
  MotionGroupOption,
  TAgentState,
} from '../types/agent'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onUnmounted, reactive, ref, watch } from 'vue'
import { getLanguageCopy } from '../i18n'
import { readAvailableMotionGroups } from '../live2d/model'
import {
  ACTION_GROUP_BINDING_SOURCE,
  AGENT_STATE_ORDER,
  APP_LANGUAGE,
  createDefaultAiTalkSettings,
  createEmptyActionGroupBindings,
  resolveActionGroupBinding,
  WINDOW_SIZE_PRESET,
} from '../types/agent'
import { MANUAL_UPDATE_WEBSITE_URL } from '../updater'

const props = defineProps<{
  bootstrap: AppBootstrap
}>()

const isSaving = ref(false)
const isScanning = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const modelMessage = ref('')
const isLoadingMotionGroups = ref(false)
const aiTalkHeadersText = ref('')

const form = reactive<AppSettings>(createFormState(props.bootstrap.settings))
const aiTalkAdvancedOpen = ref(false)
const currentScan = ref<ModelScanResult>(props.bootstrap.modelScan)
const motionGroupOptions = ref<MotionGroupOption[]>(props.bootstrap.modelScan.availableMotionGroups)
const ui = computed(() => getLanguageCopy(form.language))
const detectedMotionGroupCount = computed(() => motionGroupOptions.value.length)
const NAME_MAX_LENGTH = 16
const DEFAULT_MODEL_DIRECTORY = '/Resources/Yulia'
const AI_TALK_PROVIDER_OPTIONS = [
  { id: 'openai', label: 'OpenAI' },
  { id: 'anthropic', label: 'Anthropic' },
  { id: 'google', label: 'Google Gemini' },
  { id: 'deepseek', label: 'DeepSeek' },
  { id: 'aliyun-bailian', label: 'Alibaba Bailian / Qwen' },
  { id: 'moonshot', label: 'Moonshot Kimi' },
  { id: 'zhipu', label: 'Zhipu GLM' },
  { id: 'volcengine', label: 'Volcengine Ark / Doubao' },
  { id: 'baidu-qianfan', label: 'Baidu Qianfan / ERNIE' },
  { id: 'tencent-hunyuan', label: 'Tencent Hunyuan' },
  { id: 'minimax', label: 'MiniMax' },
  { id: 'openai-compatible', label: 'OpenAI Compatible' },
] as const
const AI_TALK_DEFAULT_MODELS: Record<string, string> = {
  openai: 'gpt-4o-mini',
  anthropic: 'claude-3-5-haiku-latest',
  google: 'gemini-1.5-flash',
  deepseek: 'deepseek-v4-pro',
  'aliyun-bailian': 'qwen-plus',
  moonshot: 'kimi-k2.6',
  zhipu: 'glm-5.1',
  volcengine: 'doubao-seed-1-6-251015',
  'baidu-qianfan': 'ernie-4.5-turbo-128k',
  'tencent-hunyuan': 'hunyuan-turbos-latest',
  minimax: 'MiniMax-M2.7',
}
const AI_TALK_DEFAULT_BASE_URLS: Record<string, string> = {
  deepseek: 'https://api.deepseek.com',
  'aliyun-bailian': 'https://dashscope.aliyuncs.com/compatible-mode/v1',
  moonshot: 'https://api.moonshot.cn/v1',
  zhipu: 'https://open.bigmodel.cn/api/paas/v4',
  volcengine: 'https://ark.cn-beijing.volces.com/api/v3',
  'baidu-qianfan': 'https://qianfan.baidubce.com/v2',
  'tencent-hunyuan': 'https://api.hunyuan.cloud.tencent.com/v1',
  minimax: 'https://api.minimaxi.com/v1',
}
let motionGroupLoadToken = 0
let isApplyingSettings = false

const currentWindow = getCurrentWindow()
const aiTalkModelPlaceholder = computed(() => (
  AI_TALK_DEFAULT_MODELS[form.aiTalk.provider] ?? ui.value.settings.aiTalkModelPlaceholder
))

watch(() => props.bootstrap.settings, (settings) => {
  applySettings(settings)
}, { deep: true, immediate: true })

watch(() => props.bootstrap.modelScan, (scan) => {
  currentScan.value = scan
  modelMessage.value = scan.validationMessage ?? ''
}, { immediate: true })

watch(
  () => [form.modelDirectory, currentScan.value.modelEntryFile] as const,
  () => {
    void loadMotionGroupOptions()
  },
  { immediate: true },
)

watch(() => form.aiTalk.provider, (provider, previousProvider) => {
  if (isApplyingSettings) {
    return
  }
  if (previousProvider) {
    storeCurrentAiTalkProviderProfile(previousProvider)
  }

  applyAiTalkProviderProfile(provider)
}, { flush: 'sync' })

onUnmounted(() => {
  motionGroupLoadToken += 1
})

function createFormState(settings: AppSettings): AppSettings {
  const aiTalk = {
    ...createDefaultAiTalkSettings(),
    ...settings.aiTalk,
    headers: {
      ...(settings.aiTalk?.headers ?? {}),
    },
    providerProfiles: cloneAiTalkProviderProfiles(settings.aiTalk?.providerProfiles ?? {}),
  }
  aiTalk.providerProfiles[aiTalk.provider] = {
    apiKey: aiTalk.apiKey,
    modelId: aiTalk.modelId,
    baseUrl: aiTalk.baseUrl,
    headers: { ...aiTalk.headers },
  }

  return {
    name: settings.name,
    language: settings.language,
    autoStart: settings.autoStart,
    modelDirectory: settings.modelDirectory,
    windowSize: settings.windowSize,
    actionGroupBindings: {
      ...createEmptyActionGroupBindings(),
      ...settings.actionGroupBindings,
    },
    aiTalk,
  }
}

function applySettings(settings: AppSettings) {
  const next = createFormState(settings)
  isApplyingSettings = true
  try {
    form.name = next.name
    form.language = next.language
    form.autoStart = next.autoStart
    form.modelDirectory = next.modelDirectory
    form.windowSize = next.windowSize
    form.aiTalk.enabled = next.aiTalk.enabled
    form.aiTalk.provider = next.aiTalk.provider
    form.aiTalk.apiKey = next.aiTalk.apiKey
    form.aiTalk.modelId = next.aiTalk.modelId
    form.aiTalk.baseUrl = next.aiTalk.baseUrl
    form.aiTalk.headers = { ...next.aiTalk.headers }
    form.aiTalk.providerProfiles = cloneAiTalkProviderProfiles(next.aiTalk.providerProfiles)
    aiTalkHeadersText.value = JSON.stringify(next.aiTalk.headers, null, 2)
    aiTalkAdvancedOpen.value = shouldOpenAiTalkAdvanced(next.aiTalk)

    for (const state of AGENT_STATE_ORDER) {
      form.actionGroupBindings[state] = next.actionGroupBindings[state]
    }
  } finally {
    isApplyingSettings = false
  }
}

function shouldOpenAiTalkAdvanced(settings: AppSettings['aiTalk']) {
  const defaultBaseUrl = AI_TALK_DEFAULT_BASE_URLS[settings.provider]
  const hasCustomBaseUrl = Boolean(settings.baseUrl?.trim())
    && normalizeAiTalkBaseUrl(settings.baseUrl) !== normalizeAiTalkBaseUrl(defaultBaseUrl)
  return settings.provider === 'openai-compatible'
    || hasCustomBaseUrl
    || Object.keys(settings.headers ?? {}).length > 0
}

function normalizeAiTalkBaseUrl(value: string | null | undefined) {
  return (value ?? '').trim().replace(/\/+$/, '').toLowerCase()
}

function cloneAiTalkProviderProfiles(profiles: Record<string, AiTalkProviderProfile>) {
  return Object.fromEntries(
    Object.entries(profiles).map(([provider, profile]) => [
      provider,
      cloneAiTalkProviderProfile(profile),
    ]),
  )
}

function cloneAiTalkProviderProfile(profile: AiTalkProviderProfile): AiTalkProviderProfile {
  return {
    apiKey: profile.apiKey ?? '',
    modelId: profile.modelId ?? '',
    baseUrl: profile.baseUrl ?? null,
    headers: { ...(profile.headers ?? {}) },
  }
}

function createDefaultAiTalkProviderProfile(provider: string): AiTalkProviderProfile {
  return {
    apiKey: '',
    modelId: AI_TALK_DEFAULT_MODELS[provider] ?? '',
    baseUrl: AI_TALK_DEFAULT_BASE_URLS[provider] ?? null,
    headers: {},
  }
}

function storeCurrentAiTalkProviderProfile(provider: string, headers = headersFromTextOrCurrent()) {
  form.aiTalk.providerProfiles[provider] = {
    apiKey: form.aiTalk.apiKey,
    modelId: form.aiTalk.modelId,
    baseUrl: form.aiTalk.baseUrl?.trim() || null,
    headers,
  }
}

function applyAiTalkProviderProfile(provider: string) {
  const profile = cloneAiTalkProviderProfile(
    form.aiTalk.providerProfiles[provider] ?? createDefaultAiTalkProviderProfile(provider),
  )

  form.aiTalk.apiKey = profile.apiKey
  form.aiTalk.modelId = profile.modelId
  form.aiTalk.baseUrl = profile.baseUrl
  form.aiTalk.headers = { ...profile.headers }
  aiTalkHeadersText.value = JSON.stringify(profile.headers, null, 2)
  aiTalkAdvancedOpen.value = shouldOpenAiTalkAdvanced({
    ...form.aiTalk,
    provider,
    baseUrl: profile.baseUrl,
    headers: profile.headers,
  })
}

function headersFromTextOrCurrent() {
  return parseAiTalkHeaders() ?? { ...form.aiTalk.headers }
}

function clearNotice() {
  errorMessage.value = ''
  successMessage.value = ''
}

function joinModelPath(basePath: string, relativePath: string) {
  return `${basePath.replace(/[\\/]+$/, '')}/${relativePath.replace(/^[\\/]+/, '')}`
}

const selectableMotionGroupOptions = computed(() => {
  const options = [...motionGroupOptions.value]
  const seen = new Set(options.map(option => option.group))

  for (const state of AGENT_STATE_ORDER) {
    const binding = form.actionGroupBindings[state]?.trim()
    if (!binding || seen.has(binding)) {
      continue
    }

    seen.add(binding)
    options.push({
      id: `${state}:${binding}`,
      group: binding,
      label: ui.value.settings.manualBindingOption(binding),
    })
  }

  return options
})

const resolvedActionGroupBindings = computed(() => {
  return Object.fromEntries(
    AGENT_STATE_ORDER.map(state => [
      state,
      resolveActionGroupBinding(state, form.actionGroupBindings, motionGroupOptions.value),
    ]),
  ) as Record<TAgentState, ReturnType<typeof resolveActionGroupBinding>>
})

async function loadMotionGroupOptions() {
  const token = ++motionGroupLoadToken
  isLoadingMotionGroups.value = true

  try {
    const modelEntryUrl = form.modelDirectory
      ? convertFileSrc(joinModelPath(form.modelDirectory, currentScan.value.modelEntryFile))
      : joinModelPath(DEFAULT_MODEL_DIRECTORY, currentScan.value.modelEntryFile)
    const options = await readAvailableMotionGroups({ modelEntryUrl })

    if (token !== motionGroupLoadToken) {
      return
    }

    motionGroupOptions.value = options.length > 0
      ? options
      : currentScan.value.availableMotionGroups
  } catch (error) {
    if (token !== motionGroupLoadToken) {
      return
    }

    console.warn('failed to load motion groups with easy-live2d', error)
    motionGroupOptions.value = currentScan.value.availableMotionGroups
  } finally {
    if (token === motionGroupLoadToken) {
      isLoadingMotionGroups.value = false
    }
  }
}

async function resetToDefaultModel() {
  clearNotice()
  form.modelDirectory = null
  isScanning.value = true

  try {
    const scan = await invoke<ModelScanResult>('scan_default_model', {
      language: form.language,
    })
    currentScan.value = scan
    modelMessage.value = ui.value.settings.switchedToDefaultModel
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    isScanning.value = false
  }
}

async function pickModelDirectory() {
  clearNotice()

  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: form.modelDirectory ?? undefined,
    title: ui.value.settings.chooseModelDirectoryTitle,
  })

  if (typeof selected !== 'string') {
    return
  }

  isScanning.value = true
  try {
    const imported = await invoke<ImportedModelResult>('import_model_directory', {
      path: selected,
      language: form.language,
    })

    form.modelDirectory = imported.importedModelDirectory
    currentScan.value = imported.modelScan
    modelMessage.value = ui.value.settings.modelValidated
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    isScanning.value = false
  }
}

async function save() {
  clearNotice()
  modelMessage.value = ''

  const trimmedName = form.name.trim()
  if (!trimmedName) {
    errorMessage.value = ui.value.settings.nameRequired
    return
  }
  if ([...trimmedName].length > NAME_MAX_LENGTH) {
    errorMessage.value = ui.value.settings.nameTooLong(NAME_MAX_LENGTH)
    return
  }

  const parsedHeaders = parseAiTalkHeaders()
  if (!parsedHeaders) {
    errorMessage.value = ui.value.settings.aiTalkHeadersInvalid
    return
  }
  storeCurrentAiTalkProviderProfile(form.aiTalk.provider, parsedHeaders)

  isSaving.value = true

  try {
    await invoke<AppBootstrap>('save_settings', {
      settings: {
        ...form,
        name: trimmedName,
        actionGroupBindings: { ...form.actionGroupBindings },
        aiTalk: {
          ...form.aiTalk,
          headers: parsedHeaders,
          baseUrl: form.aiTalk.baseUrl?.trim() || null,
          apiKey: form.aiTalk.apiKey.trim(),
          modelId: form.aiTalk.modelId.trim(),
          providerProfiles: sanitizeAiTalkProviderProfiles(form.aiTalk.providerProfiles),
        },
      },
    })

    successMessage.value = ui.value.settings.saveSuccess
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    isSaving.value = false
  }
}

function cancel() {
  void currentWindow.close()
}

async function openOfficialWebsite() {
  try {
    await openUrl(MANUAL_UPDATE_WEBSITE_URL)
  } catch (error) {
    console.warn('failed to open official website', error)
  }
}

function setActionGroupBinding(state: TAgentState, value: string) {
  const trimmedValue = value.trim()
  form.actionGroupBindings[state] = trimmedValue || null
}

function actionGroupBindingStatus(state: TAgentState) {
  const resolved = resolvedActionGroupBindings.value[state]

  if (resolved.source === ACTION_GROUP_BINDING_SOURCE.MANUAL && resolved.group) {
    return ui.value.settings.manualBindingStatus(resolved.group)
  }

  if (resolved.source === ACTION_GROUP_BINDING_SOURCE.AUTO && resolved.group) {
    return ui.value.settings.autoBindingStatus(resolved.group)
  }

  return ui.value.settings.unresolvedBindingStatus
}

function parseAiTalkHeaders() {
  const raw = aiTalkHeadersText.value.trim()
  if (!raw) {
    return {}
  }

  try {
    const parsed = JSON.parse(raw)
    if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
      return null
    }

    const entries = Object.entries(parsed)
    if (entries.some(([key, value]) => !key.trim() || typeof value !== 'string')) {
      return null
    }

    return Object.fromEntries(
      entries
        .filter(([, value]) => (value as string).trim())
        .map(([key, value]) => [key.trim(), (value as string).trim()]),
    )
  } catch {
    return null
  }
}

function sanitizeAiTalkProviderProfiles(profiles: Record<string, AiTalkProviderProfile>) {
  return Object.fromEntries(
    Object.entries(profiles)
      .map(([provider, profile]) => [
        provider.trim(),
        {
          apiKey: profile.apiKey.trim(),
          modelId: profile.modelId.trim(),
          baseUrl: profile.baseUrl?.trim() || null,
          headers: sanitizeAiTalkHeaders(profile.headers),
        },
      ] as const)
      .filter(([provider]) => provider),
  )
}

function sanitizeAiTalkHeaders(headers: Record<string, string>) {
  return Object.fromEntries(
    Object.entries(headers)
      .map(([key, value]) => [key.trim(), value.trim()] as const)
      .filter(([key, value]) => key && value),
  )
}
</script>

<template>
  <div class="settings">
    <header class="settings__hero">
      <p class="settings__eyebrow">
        {{ ui.settings.eyebrow }}
      </p>
      <h1>{{ ui.settings.title }}</h1>
      <p class="settings__description">
        {{ ui.settings.description }}
      </p>
      <p class="settings__version">
        <span>{{ ui.settings.versionLabel }}</span>
        <strong>{{ props.bootstrap.appVersion }}</strong>
      </p>
    </header>

    <section class="settings__panel">
      <label class="field field--switch">
        <span>
          <strong>{{ ui.settings.autoStartLabel }}</strong>
          <small>{{ ui.settings.autoStartHint }}</small>
        </span>
        <input
          v-model="form.autoStart"
          type="checkbox"
        >
      </label>

      <div class="field ai-talk">
        <label class="field--switch">
          <span>
            <strong>{{ ui.settings.aiTalkLabel }}</strong>
            <small>{{ ui.settings.aiTalkHint }}</small>
          </span>
          <input
            v-model="form.aiTalk.enabled"
            type="checkbox"
          >
        </label>

        <div
          v-if="form.aiTalk.enabled"
          class="ai-talk__grid"
        >
          <label class="field">
            <span class="field__label">{{ ui.settings.aiTalkProviderLabel }}</span>
            <select
              v-model="form.aiTalk.provider"
              class="field__select"
            >
              <option
                v-for="provider in AI_TALK_PROVIDER_OPTIONS"
                :key="provider.id"
                :value="provider.id"
              >
                {{ provider.label }}
              </option>
            </select>
          </label>

          <label class="field">
            <span class="field__label">{{ ui.settings.aiTalkModelLabel }}</span>
            <input
              v-model="form.aiTalk.modelId"
              class="field__input"
              type="text"
              :placeholder="aiTalkModelPlaceholder"
            >
          </label>

          <label class="field ai-talk__full">
            <span class="field__label">{{ ui.settings.aiTalkApiKeyLabel }}</span>
            <input
              v-model="form.aiTalk.apiKey"
              class="field__input"
              type="password"
              autocomplete="off"
              :placeholder="ui.settings.aiTalkApiKeyPlaceholder"
            >
            <small class="field__hint">{{ ui.settings.aiTalkApiKeyHint }}</small>
          </label>

          <button
            class="advanced-toggle ai-talk__full"
            type="button"
            :aria-expanded="aiTalkAdvancedOpen"
            @click="aiTalkAdvancedOpen = !aiTalkAdvancedOpen"
          >
            <span>
              <strong>{{ ui.settings.aiTalkAdvancedLabel }}</strong>
              <small>{{ ui.settings.aiTalkAdvancedHint }}</small>
            </span>
            <span
              class="advanced-toggle__chevron"
              :class="{ 'advanced-toggle__chevron--open': aiTalkAdvancedOpen }"
              aria-hidden="true"
            />
          </button>

          <div
            v-if="aiTalkAdvancedOpen"
            class="ai-talk__advanced ai-talk__full"
          >
            <label class="field">
              <span class="field__label">{{ ui.settings.aiTalkBaseUrlLabel }}</span>
              <input
                v-model="form.aiTalk.baseUrl"
                class="field__input"
                type="text"
                :placeholder="ui.settings.aiTalkBaseUrlPlaceholder"
              >
              <small class="field__hint">{{ ui.settings.aiTalkBaseUrlHint }}</small>
            </label>

            <label class="field">
              <span class="field__label">{{ ui.settings.aiTalkHeadersLabel }}</span>
              <textarea
                v-model="aiTalkHeadersText"
                class="field__textarea"
                spellcheck="false"
                rows="4"
                :placeholder="ui.settings.aiTalkHeadersPlaceholder"
              />
              <small class="field__hint">{{ ui.settings.aiTalkHeadersHint }}</small>
            </label>
          </div>
        </div>
      </div>

      <label class="field">
        <span class="field__label">{{ ui.settings.languageLabel }}</span>
        <select
          v-model="form.language"
          class="field__select"
        >
          <option :value="APP_LANGUAGE.ENGLISH">
            English
          </option>
          <option :value="APP_LANGUAGE.CHINESE">
            中文
          </option>
          <option :value="APP_LANGUAGE.JAPANESE">
            日本語
          </option>
        </select>
      </label>

      <label class="field">
        <span class="field__label">{{ ui.settings.nameLabel }}</span>
        <input
          v-model="form.name"
          class="field__input"
          :maxlength="NAME_MAX_LENGTH"
          type="text"
          :placeholder="ui.settings.namePlaceholder"
        >
        <small class="field__hint">
          {{ ui.settings.nameCount([...form.name].length, NAME_MAX_LENGTH) }}
        </small>
      </label>

      <div class="field">
        <span class="field__label">{{ ui.settings.uploadModelLabel }}</span>
        <div class="model-picker">
          <button
            class="button"
            :disabled="isScanning"
            type="button"
            @click="pickModelDirectory"
          >
            {{ isScanning ? ui.settings.validating : ui.settings.chooseDirectory }}
          </button>
          <button
            class="button button--secondary"
            type="button"
            @click="resetToDefaultModel"
          >
            {{ ui.settings.useDefaultModel }}
          </button>
        </div>
        <p class="field__path">
          {{ form.modelDirectory || ui.settings.builtInModelPath }}
        </p>
        <p
          v-if="modelMessage"
          class="field__hint"
        >
          {{ modelMessage }}
        </p>
      </div>

      <div class="field">
        <span class="field__label">{{ ui.settings.windowSizeLabel }}</span>
        <div class="size-grid">
          <label class="choice">
            <input
              v-model="form.windowSize"
              :value="WINDOW_SIZE_PRESET.TINY"
              type="radio"
            >
            <span>{{ ui.windowSizeLabels[WINDOW_SIZE_PRESET.TINY] }}</span>
          </label>
          <label class="choice">
            <input
              v-model="form.windowSize"
              :value="WINDOW_SIZE_PRESET.SMALL"
              type="radio"
            >
            <span>{{ ui.windowSizeLabels[WINDOW_SIZE_PRESET.SMALL] }}</span>
          </label>
          <label class="choice">
            <input
              v-model="form.windowSize"
              :value="WINDOW_SIZE_PRESET.MEDIUM"
              type="radio"
            >
            <span>{{ ui.windowSizeLabels[WINDOW_SIZE_PRESET.MEDIUM] }}</span>
          </label>
          <label class="choice">
            <input
              v-model="form.windowSize"
              :value="WINDOW_SIZE_PRESET.LARGE"
              type="radio"
            >
            <span>{{ ui.windowSizeLabels[WINDOW_SIZE_PRESET.LARGE] }}</span>
          </label>
        </div>
      </div>

      <div class="field">
        <span class="field__label">{{ ui.settings.websiteLabel }}</span>
        <div class="model-picker">
          <button
            class="button button--secondary"
            type="button"
            @click="openOfficialWebsite"
          >
            {{ ui.updater.openWebsite }}
          </button>
        </div>
        <p class="field__path">
          {{ MANUAL_UPDATE_WEBSITE_URL }}
        </p>
      </div>

      <div class="field">
        <span class="field__label">{{ ui.settings.actionGroupBindingLabel }}</span>
        <div class="binding-list">
          <label
            v-for="state in AGENT_STATE_ORDER"
            :key="state"
            class="binding-row"
          >
            <span class="binding-row__meta">
              <strong>{{ ui.stateLabels[state] }}</strong>
              <small
                class="binding-row__status"
                :class="[
                  `binding-row__status--${resolvedActionGroupBindings[state].source}`,
                ]"
              >
                {{ actionGroupBindingStatus(state) }}
              </small>
            </span>
            <select
              class="field__select"
              :value="form.actionGroupBindings[state] ?? ''"
              :disabled="isLoadingMotionGroups"
              @change="setActionGroupBinding(state, ($event.target as HTMLSelectElement).value)"
            >
              <option value="">
                {{ isLoadingMotionGroups ? ui.settings.loadingActionGroups : ui.settings.noBinding }}
              </option>
              <option
                v-for="option in selectableMotionGroupOptions"
                :key="option.id"
                :value="option.group"
              >
                {{ option.label }}
              </option>
            </select>
          </label>
        </div>
        <p class="field__hint">
          {{
            detectedMotionGroupCount > 0
              ? ui.settings.actionGroupOptionsLoaded(detectedMotionGroupCount)
              : ui.settings.noActionGroupsFound
          }}
        </p>
      </div>

      <p
        v-if="errorMessage"
        class="notice notice--error"
      >
        {{ errorMessage }}
      </p>
      <p
        v-if="successMessage"
        class="notice notice--success"
      >
        {{ successMessage }}
      </p>
    </section>

    <footer class="settings__footer">
      <button
        class="button button--secondary"
        type="button"
        @click="cancel"
      >
        {{ ui.settings.cancel }}
      </button>
      <button
        class="button"
        :disabled="isSaving"
        type="button"
        @click="save"
      >
        {{ isSaving ? ui.settings.saving : ui.settings.save }}
      </button>
    </footer>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  width: 100%;
  min-height: 100%;
  padding: 28px 24px 22px;
  box-sizing: border-box;
  color: #203031;
}

.settings__hero h1 {
  margin: 6px 0 0;
  font-size: 30px;
  line-height: 1.1;
}

.settings__eyebrow {
  margin: 0;
  color: #7d5f41;
  font-size: 12px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.settings__description {
  margin: 12px 0 0;
  color: #4f6362;
  font-size: 14px;
  line-height: 1.6;
}

.settings__version {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin: 14px 0 0;
  padding: 8px 12px;
  border: 1px solid rgba(70, 107, 105, 0.16);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.62);
  color: #5f7472;
  font-size: 12px;
  width: fit-content;
}

.settings__version strong {
  color: #28413f;
  font-size: 13px;
}

.settings__panel {
  margin-top: 22px;
  padding: 18px;
  border: 1px solid rgba(62, 95, 93, 0.14);
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.8);
  box-shadow: 0 18px 48px rgba(38, 59, 58, 0.12);
  backdrop-filter: blur(14px);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field + .field {
  margin-top: 18px;
}

.field--switch {
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
}

.field--switch small {
  display: block;
  margin-top: 4px;
  color: #657c7b;
}

.field__label {
  font-size: 13px;
  font-weight: 700;
  color: #33514f;
}

.field__input,
.field__select,
.field__textarea {
  width: 100%;
  min-height: 42px;
  padding: 0 14px;
  border: 1px solid rgba(70, 107, 105, 0.18);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.88);
  color: #233635;
  font-size: 14px;
  box-sizing: border-box;
}

.field__textarea {
  min-height: 92px;
  padding: 12px 14px;
  resize: vertical;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  line-height: 1.45;
}

.field__hint,
.field__path {
  margin: 0;
  color: #617a78;
  font-size: 12px;
  line-height: 1.5;
}

.field__path {
  word-break: break-all;
}

.ai-talk {
  padding: 14px;
  border: 1px solid rgba(70, 107, 105, 0.14);
  border-radius: 18px;
  background: rgba(247, 250, 248, 0.72);
}

.ai-talk__grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.ai-talk__grid .field + .field {
  margin-top: 0;
}

.ai-talk__full {
  grid-column: 1 / -1;
}

.ai-talk__advanced {
  display: grid;
  gap: 12px;
  padding: 12px;
  border: 1px solid rgba(70, 107, 105, 0.12);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.5);
}

.advanced-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  min-height: 50px;
  padding: 10px 12px;
  border: 1px solid rgba(70, 107, 105, 0.14);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.58);
  color: #33514f;
  text-align: left;
  cursor: pointer;
}

.advanced-toggle strong,
.advanced-toggle small {
  display: block;
}

.advanced-toggle strong {
  font-size: 13px;
}

.advanced-toggle small {
  margin-top: 4px;
  color: #617a78;
  font-size: 12px;
  line-height: 1.4;
}

.advanced-toggle__chevron {
  width: 8px;
  height: 8px;
  border-right: 2px solid #617a78;
  border-bottom: 2px solid #617a78;
  transform: rotate(45deg);
  transition: transform 0.18s ease;
}

.advanced-toggle__chevron--open {
  transform: rotate(225deg);
}

.model-picker,
.settings__footer,
.size-grid {
  display: flex;
  gap: 10px;
}

.size-grid {
  flex-wrap: wrap;
}

.choice {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border: 1px solid rgba(70, 107, 105, 0.16);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.7);
  font-size: 13px;
}

.binding-list {
  display: grid;
  gap: 10px;
}

.binding-row {
  display: grid;
  grid-template-columns: 1fr 1.3fr;
  gap: 12px;
  align-items: start;
  font-size: 13px;
}

.binding-row__meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.binding-row__meta strong {
  color: #33514f;
  font-size: 13px;
}

.binding-row__status {
  color: #617a78;
  font-size: 12px;
  line-height: 1.4;
}

.binding-row__status--manual {
  color: #7a4a1e;
}

.binding-row__status--auto {
  color: #2f6860;
}

.binding-row__status--unresolved {
  color: #8a5b35;
}

.notice {
  margin: 18px 0 0;
  padding: 12px 14px;
  border-radius: 14px;
  font-size: 13px;
}

.notice--error {
  background: rgba(188, 92, 92, 0.12);
  color: #8a3434;
}

.notice--success {
  background: rgba(78, 160, 118, 0.14);
  color: #24563a;
}

.settings__footer {
  justify-content: flex-end;
  margin-top: auto;
  padding-top: 18px;
}

.button {
  min-height: 42px;
  padding: 0 18px;
  border: 0;
  border-radius: 999px;
  background: linear-gradient(135deg, #e58c52, #d66a4a);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  box-shadow: 0 10px 24px rgba(214, 106, 74, 0.22);
}

.button:disabled {
  cursor: wait;
  opacity: 0.68;
}

.button--secondary {
  background: rgba(255, 255, 255, 0.86);
  color: #345250;
  box-shadow: none;
  border: 1px solid rgba(70, 107, 105, 0.16);
}
</style>
