<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, reactive, ref, watch } from 'vue'
import {
  AGENT_STATE_LABEL,
  AGENT_STATE_ORDER,
  WINDOW_SIZE_PRESET,
  createEmptyActionGroupBindings,
} from '../types/agent'
import type { AppBootstrap, AppSettings, ModelScanResult, TAgentState } from '../types/agent'

const props = defineProps<{
  bootstrap: AppBootstrap
}>()

const isSaving = ref(false)
const isScanning = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const modelMessage = ref('')

const form = reactive<AppSettings>(createFormState(props.bootstrap.settings))
const currentScan = ref<ModelScanResult>(props.bootstrap.modelScan)

const motionGroupOptions = computed(() => currentScan.value.availableMotionGroups)
const currentWindow = getCurrentWindow()

watch(() => props.bootstrap.settings, (settings) => {
  applySettings(settings)
}, { deep: true, immediate: true })

watch(() => props.bootstrap.modelScan, (scan) => {
  currentScan.value = scan
  modelMessage.value = scan.validationMessage ?? ''
}, { immediate: true })

function createFormState(settings: AppSettings): AppSettings {
  return {
    name: settings.name,
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

async function resetToDefaultModel() {
  clearNotice()
  form.modelDirectory = null
  isScanning.value = true

  try {
    const scan = await invoke<ModelScanResult>('scan_default_model')
    currentScan.value = scan
    modelMessage.value = '将切换回内置默认模型。'

    const availableIds = new Set(scan.availableMotionGroups.map(option => option.id))
    for (const state of AGENT_STATE_ORDER) {
      const binding = form.actionGroupBindings[state]
      if (binding && !availableIds.has(binding)) {
        form.actionGroupBindings[state] = null
      }
    }
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
    title: '选择 Live2D 模型目录',
  })

  if (typeof selected !== 'string') {
    return
  }

  isScanning.value = true
  try {
    const scan = await invoke<ModelScanResult>('scan_model_directory', {
      path: selected,
    })

    form.modelDirectory = selected
    currentScan.value = scan
    modelMessage.value = '模型目录校验通过。'

    const availableIds = new Set(scan.availableMotionGroups.map(option => option.id))
    for (const state of AGENT_STATE_ORDER) {
      const binding = form.actionGroupBindings[state]
      if (binding && !availableIds.has(binding)) {
        form.actionGroupBindings[state] = null
      }
    }
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
    errorMessage.value = 'Name 不能为空。'
    return
  }
  if ([...trimmedName].length > 16) {
    errorMessage.value = 'Name 最多支持 16 个字符。'
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

    successMessage.value = '设置已保存并立即生效。'
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
  form.actionGroupBindings[state] = value || null
}
</script>

<template>
  <div class="settings">
    <header class="settings__hero">
      <p class="settings__eyebrow">
        Copiwaifu
      </p>
      <h1>桌宠设置</h1>
      <p class="settings__description">
        控制名字、模型、尺寸和状态动作组绑定。保存后主窗口会立即同步。
      </p>
    </header>

    <section class="settings__panel">
      <label class="field field--switch">
        <span>
          <strong>Auto Start</strong>
          <small>保存后同步系统开机自启状态。</small>
        </span>
        <input
          v-model="form.autoStart"
          type="checkbox"
        >
      </label>

      <label class="field">
        <span class="field__label">Name</span>
        <input
          v-model="form.name"
          class="field__input"
          maxlength="16"
          type="text"
          placeholder="Copiwaifu"
        >
        <small class="field__hint">
          当前 {{ [...form.name].length }}/16
        </small>
      </label>

      <div class="field">
        <span class="field__label">Upload Model</span>
        <div class="model-picker">
          <button
            class="button"
            :disabled="isScanning"
            type="button"
            @click="pickModelDirectory"
          >
            {{ isScanning ? '校验中...' : '选择目录' }}
          </button>
          <button
            class="button button--secondary"
            type="button"
            @click="resetToDefaultModel"
          >
            使用默认模型
          </button>
        </div>
        <p class="field__path">
          {{ form.modelDirectory || '当前使用内置 Hiyori 模型' }}
        </p>
        <p
          v-if="modelMessage"
          class="field__hint"
        >
          {{ modelMessage }}
        </p>
      </div>

      <div class="field">
        <span class="field__label">Window Size</span>
        <div class="size-grid">
          <label class="choice">
            <input
              v-model="form.windowSize"
              :value="WINDOW_SIZE_PRESET.SMALL"
              type="radio"
            >
            <span>小</span>
          </label>
          <label class="choice">
            <input
              v-model="form.windowSize"
              :value="WINDOW_SIZE_PRESET.MEDIUM"
              type="radio"
            >
            <span>中</span>
          </label>
          <label class="choice">
            <input
              v-model="form.windowSize"
              :value="WINDOW_SIZE_PRESET.LARGE"
              type="radio"
            >
            <span>大</span>
          </label>
        </div>
      </div>

      <div class="field">
        <span class="field__label">Action Group Binding / 动作组绑定</span>
        <div class="binding-list">
          <label
            v-for="state in AGENT_STATE_ORDER"
            :key="state"
            class="binding-row"
          >
            <span>{{ AGENT_STATE_LABEL[state] }}</span>
            <select
              class="field__select"
              :value="form.actionGroupBindings[state] ?? ''"
              @change="setActionGroupBinding(state, ($event.target as HTMLSelectElement).value)"
            >
              <option value="">
                不绑定
              </option>
              <option
                v-for="option in motionGroupOptions"
                :key="option.id"
                :value="option.id"
              >
                {{ option.label }}
              </option>
            </select>
          </label>
        </div>
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
        Cancel
      </button>
      <button
        class="button"
        :disabled="isSaving"
        type="button"
        @click="save"
      >
        {{ isSaving ? 'Saving...' : 'Save' }}
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
