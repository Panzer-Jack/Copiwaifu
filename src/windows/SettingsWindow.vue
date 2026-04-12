<script setup lang="ts">
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { CubismSetting, Live2DSprite } from 'easy-live2d'
import { Application, Ticker } from 'pixi.js'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onUnmounted, reactive, ref, watch } from 'vue'
import { getLanguageCopy } from '../i18n'
import {
  AGENT_STATE_ORDER,
  APP_LANGUAGE,
  WINDOW_SIZE_PRESET,
  createEmptyActionGroupBindings,
} from '../types/agent'
import type {
  AppBootstrap,
  AppSettings,
  ImportedModelResult,
  ModelScanResult,
  MotionGroupOption,
  TAgentState,
} from '../types/agent'

const props = defineProps<{
  bootstrap: AppBootstrap
}>()

const isSaving = ref(false)
const isScanning = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const modelMessage = ref('')
const isLoadingMotionGroups = ref(false)

const form = reactive<AppSettings>(createFormState(props.bootstrap.settings))
const currentScan = ref<ModelScanResult>(props.bootstrap.modelScan)
const motionGroupOptions = ref<MotionGroupOption[]>(props.bootstrap.modelScan.availableMotionGroups)
const ui = computed(() => getLanguageCopy(form.language))
const NAME_MAX_LENGTH = 16
const DEFAULT_MODEL_DIRECTORY = '/Resources/Yulia'
let motionGroupLoadToken = 0

const currentWindow = getCurrentWindow()

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

onUnmounted(() => {
  motionGroupLoadToken += 1
})

function createFormState(settings: AppSettings): AppSettings {
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
  }
}

function applySettings(settings: AppSettings) {
  const next = createFormState(settings)
  form.name = next.name
  form.language = next.language
  form.autoStart = next.autoStart
  form.modelDirectory = next.modelDirectory
  form.windowSize = next.windowSize

  for (const state of AGENT_STATE_ORDER) {
    form.actionGroupBindings[state] = next.actionGroupBindings[state]
  }
}

function clearNotice() {
  errorMessage.value = ''
  successMessage.value = ''
}

function buildMotionGroupOptions(groups: string[]): MotionGroupOption[] {
  const seen = new Set<string>()
  const options: MotionGroupOption[] = []

  for (const group of groups) {
    const trimmedGroup = group.trim()
    if (!trimmedGroup || seen.has(trimmedGroup)) {
      continue
    }

    seen.add(trimmedGroup)
    options.push({
      id: trimmedGroup,
      group: trimmedGroup,
      label: trimmedGroup,
    })
  }

  return options
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
      id: binding,
      group: binding,
      label: binding,
    })
  }

  return options
})

async function createMotionReaderSprite() {
  if (form.modelDirectory) {
    const modelEntryPath = joinModelPath(form.modelDirectory, currentScan.value.modelEntryFile)
    const modelEntryUrl = convertFileSrc(modelEntryPath)
    const response = await fetch(modelEntryUrl)

    if (!response.ok) {
      throw new Error(`failed_to_read_model_json: ${response.status}`)
    }

    const modelJSON = await response.json()
    const modelSetting = new CubismSetting({ modelJSON })

    modelSetting.redirectPath(({ file }) => {
      return convertFileSrc(joinModelPath(form.modelDirectory as string, file))
    })

    return new Live2DSprite({
      modelSetting,
      ticker: Ticker.shared,
    })
  }

  return new Live2DSprite({
    modelPath: joinModelPath(DEFAULT_MODEL_DIRECTORY, currentScan.value.modelEntryFile),
    ticker: Ticker.shared,
  })
}

async function readMotionGroupOptionsWithEasyLive2D() {
  const app = new Application()
  let sprite: Live2DSprite | null = null

  await app.init({
    canvas: document.createElement('canvas'),
    width: 1,
    height: 1,
    resolution: 1,
    autoDensity: true,
    backgroundAlpha: 0,
  })

  try {
    sprite = await createMotionReaderSprite()
    app.stage.addChild(sprite as any)
    await sprite.ready
    return buildMotionGroupOptions(sprite.getMotions().map(motion => motion.group))
  }
  finally {
    if (sprite) {
      app.stage.removeChild(sprite as any)
      sprite.destroy()
    }
    app.destroy(true)
  }
}

async function loadMotionGroupOptions() {
  const token = ++motionGroupLoadToken
  isLoadingMotionGroups.value = true

  try {
    const options = await readMotionGroupOptionsWithEasyLive2D()
    if (token !== motionGroupLoadToken) {
      return
    }

    motionGroupOptions.value = options.length > 0
      ? options
      : currentScan.value.availableMotionGroups
  }
  catch (error) {
    if (token !== motionGroupLoadToken) {
      return
    }

    console.warn('failed to load motion groups with easy-live2d', error)
    motionGroupOptions.value = currentScan.value.availableMotionGroups
  }
  finally {
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
  }
  catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
  finally {
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
  }
  catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
  finally {
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

  isSaving.value = true

  try {
    await invoke<AppBootstrap>('save_settings', {
      settings: {
        ...form,
        name: trimmedName,
        actionGroupBindings: { ...form.actionGroupBindings },
      },
    })

    successMessage.value = ui.value.settings.saveSuccess
  }
  catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
  finally {
    isSaving.value = false
  }
}

function cancel() {
  void currentWindow.close()
}

function setActionGroupBinding(state: TAgentState, value: string) {
  const trimmedValue = value.trim()
  form.actionGroupBindings[state] = trimmedValue || null
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
        <span class="field__label">{{ ui.settings.actionGroupBindingLabel }}</span>
        <div class="binding-list">
          <label
            v-for="state in AGENT_STATE_ORDER"
            :key="state"
            class="binding-row"
          >
            <span>{{ ui.stateLabels[state] }}</span>
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
            selectableMotionGroupOptions.length > 0
              ? ui.settings.actionGroupOptionsLoaded(selectableMotionGroupOptions.length)
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
.field__select {
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
  align-items: center;
  font-size: 13px;
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
