import { listen as tauriListen } from '@tauri-apps/api/event'
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

const sourceEl = document.querySelector('#source')
const targetEl = document.querySelector('#target')
const statusEl = document.querySelector('#status')
const viewTitleEl = document.querySelector('#view-title')
const viewSubtitleEl = document.querySelector('#view-subtitle')
const providerEl = document.querySelector('#provider')
const shortcutEl = document.querySelector('#shortcut')
const autostartInput = document.querySelector('#autostart')
const translateButton = document.querySelector('#translate')
const simulateButton = document.querySelector('#simulate')
const simulateSelectionViewButton = document.querySelector('#simulate-selection-view')
const refreshButton = document.querySelector('#refresh')
const manualInput = document.querySelector('#manual-input')
const sourceLangEl = document.querySelector('#source-lang')
const targetLangEl = document.querySelector('#target-lang')
const swapLangButton = document.querySelector('#swap-lang')
const clearInputButton = document.querySelector('#clear-input')
const copyTargetButton = document.querySelector('#copy-target')
const inputCountEl = document.querySelector('#input-count')
const autoTranslateToggle = document.querySelector('#auto-translate-toggle')
const languageResultEl = document.querySelector('#language-result')
const configureAiButton = document.querySelector('#configure-ai')
const aiConfigPanel = document.querySelector('#ai-config')
const closeAiButton = document.querySelector('#close-ai')
const saveAiConfigButton = document.querySelector('#save-ai-config')
const providerConfigNoteEl = document.querySelector('#provider-config-note')
const activeProviderInput = document.querySelector('#active-provider')
const configProviderInput = document.querySelector('#config-provider')
const providerNameInput = document.querySelector('#provider-name-input')
const providerProtocolInput = document.querySelector('#provider-protocol')
const aiBaseUrlInput = document.querySelector('#ai-base-url')
const aiModelInput = document.querySelector('#ai-model')
const aiKeyInput = document.querySelector('#ai-key')
const providerAgentInput = document.querySelector('#provider-agent')
const customProviderFields = [...document.querySelectorAll('.custom-provider-field')]
const navButtons = [...document.querySelectorAll('.nav-item')]
const viewSections = [...document.querySelectorAll('[data-view-section]')]
const providerStrip = document.querySelector('.provider-strip')
const systemPanel = document.querySelector('.system-panel')
const providerLangpairEl = document.querySelector('#provider-langpair')
const providerAiStateEl = document.querySelector('#provider-ai-state')
const settingsAutostartEl = document.querySelector('#settings-autostart')
const settingsShortcutInput = document.querySelector('#settings-shortcut')
const settingsMainShortcutInput = document.querySelector('#settings-main-shortcut')
const settingsStartupModeInput = document.querySelector('#settings-startup-mode')
const settingsAutostartToggle = document.querySelector('#settings-autostart-toggle')
const saveSettingsButton = document.querySelector('#save-settings')
const settingsNoteEl = document.querySelector('#settings-note')
const settingsThemeInput = document.querySelector('#settings-theme')
const windowActionButtons = [...document.querySelectorAll('[data-window-action]')]
const selectionShortcutEl = document.querySelector('#selection-shortcut')
const shortcutBadgeEl = document.querySelector('.shortcut-badge')
const toastEl = document.querySelector('#toast')
const isPopupPage = document.body.classList.contains('popup-page')

let aiConfigOpen = false
let autoTranslateTimer = null
let translationRequestId = 0
let activeTranslationRequestId = 0
let isTranslating = false
let queuedAutoTranslate = false
let autoTranslateEnabled = true
let lastTranslatedSignature = ''
let activeTranslationSignature = ''
let queuedAutoSignature = ''
let providerState = { active_provider_id: 'mymemory', providers: [] }
const customSelects = new WeakMap()

const AUTO_TRANSLATE_DEFAULT_DELAY_MS = 1100
const AUTO_TRANSLATE_PASTE_DELAY_MS = 250
const AUTO_TRANSLATE_SHORT_DELAY_MS = 800
const AUTO_TRANSLATE_LONG_DELAY_MS = 1500
const AUTO_TRANSLATE_LEAVE_DELAY_MS = 150
const QUEUED_TRANSLATE_DELAY_MS = 180
const THEME_STORAGE_KEY = 'ai-translate-theme'
const DEFAULT_PROVIDER_ID = 'mymemory'
const CUSTOM_PROVIDER_VALUE = '__custom_openai__'
const PROVIDER_PRESETS = {
  mymemory: {
    name: 'MyMemory 公共接口',
    protocol: 'mymemory',
    base_url: '',
    model: '',
  },
  deepseek: {
    name: 'DeepSeek',
    protocol: 'openai',
    base_url: 'https://api.deepseek.com',
    model: 'deepseek-v4-flash',
  },
}

const MODIFIER_LABELS = [
  ['ctrlKey', 'Ctrl'],
  ['altKey', 'Alt'],
  ['shiftKey', 'Shift'],
]

const viewCopy = {
  translate: {
    title: '翻译工作台',
    subtitle: '输入或粘贴文本会自动翻译；在任意应用选中文字后按 Alt + D 使用划词悬浮窗。',
  },
  selection: {
    title: '划词翻译',
    subtitle: '用于测试全局快捷键、剪贴板读取和鼠标附近悬浮窗的完整链路。',
  },
  provider: {
    title: '翻译服务',
    subtitle: '配置 DeepSeek、OpenAI Compatible 或公共翻译服务。',
  },
  settings: {
    title: '应用设置',
    subtitle: '管理启动、快捷键和运行状态。当前阶段只开放已经接通后端的设置。',
  },
}

function setStatus(text) {
  if (statusEl) statusEl.textContent = text
}

function getTranslationSignature(text = manualInput?.value.trim() || '') {
  const sourceLang = sourceLangEl?.value || 'auto'
  const targetLang = targetLangEl?.value || 'zh-CN'
  return `${providerState.active_provider_id}\n${sourceLang}\n${targetLang}\n${text}`
}

function setTranslationBusy(isBusy, label = '正在翻译...') {
  document.body.classList.toggle('is-translating', isBusy)
  if (translateButton) {
    translateButton.disabled = isBusy
    translateButton.textContent = isBusy ? '翻译中...' : '立即翻译'
  }
  if (targetEl) {
    targetEl.classList.toggle('loading', isBusy)
    if (isBusy) targetEl.textContent = label
  }
}

function showToast(message, variant = 'success') {
  if (!toastEl) return
  toastEl.textContent = message
  toastEl.hidden = false
  toastEl.classList.toggle('error', variant === 'error')
  window.requestAnimationFrame(() => toastEl.classList.add('show'))
  window.clearTimeout(showToast.timer)
  showToast.timer = window.setTimeout(() => {
    toastEl.classList.remove('show')
    window.setTimeout(() => {
      toastEl.hidden = true
    }, 180)
  }, 2200)
}

function closeCustomSelects(except) {
  customSelects.forEach?.((custom) => {
    if (custom !== except) custom.root.classList.remove('open')
  })

  document.querySelectorAll('.custom-select.open').forEach((root) => {
    if (root !== except?.root) root.classList.remove('open')
  })
}

function refreshCustomSelect(select) {
  const custom = customSelects.get(select)
  if (!custom) return

  const selected = select.selectedOptions?.[0] || select.options?.[select.selectedIndex]
  custom.buttonText.textContent = selected?.textContent?.trim() || ''
  custom.menu.innerHTML = ''

  Array.from(select.options).forEach((option) => {
    const item = document.createElement('button')
    item.type = 'button'
    item.className = 'custom-option'
    item.textContent = option.textContent
    item.disabled = option.disabled
    item.classList.toggle('selected', option.value === select.value)
    item.addEventListener('click', () => {
      if (option.disabled) return
      select.value = option.value
      select.dispatchEvent(new Event('change', { bubbles: true }))
      custom.root.classList.remove('open')
      refreshCustomSelect(select)
    })
    custom.menu.append(item)
  })
}

function enhanceCustomSelect(select) {
  if (!select || customSelects.has(select)) return

  const root = document.createElement('div')
  root.className = 'custom-select'
  const button = document.createElement('button')
  button.type = 'button'
  button.className = 'custom-select-button'
  const buttonText = document.createElement('span')
  buttonText.className = 'custom-select-text'
  const chevron = document.createElement('span')
  chevron.className = 'custom-select-chevron'
  chevron.textContent = '⌄'
  button.append(buttonText, chevron)
  const menu = document.createElement('div')
  menu.className = 'custom-select-menu'
  root.append(button, menu)

  select.classList.add('native-select-hidden')
  select.insertAdjacentElement('afterend', root)

  const custom = { root, button, buttonText, menu }
  customSelects.set(select, custom)

  button.addEventListener('click', (event) => {
    event.preventDefault()
    event.stopPropagation()
    const willOpen = !root.classList.contains('open')
    closeCustomSelects(custom)
    root.classList.toggle('open', willOpen)
  })

  select.addEventListener('change', () => refreshCustomSelect(select))
  new MutationObserver(() => refreshCustomSelect(select)).observe(select, {
    childList: true,
    subtree: true,
    attributes: true,
  })
  refreshCustomSelect(select)
}

function enhanceCustomSelects() {
  document.querySelectorAll('select').forEach(enhanceCustomSelect)
}

function applyTheme(theme, options = {}) {
  const nextTheme = ['dark', 'compact', 'light'].includes(theme) ? theme : 'dark'
  document.documentElement.dataset.theme = nextTheme
  if (options.persist !== false) {
    window.localStorage.setItem(THEME_STORAGE_KEY, nextTheme)
  }
  if (settingsThemeInput) settingsThemeInput.value = nextTheme
  refreshCustomSelect(settingsThemeInput)
}

function normalizeShortcutInput(value) {
  const parts = value
    .split('+')
    .map((part) => part.trim())
    .filter(Boolean)

  const modifiers = []
  let key = ''

  parts.forEach((part) => {
    const lower = part.toLowerCase()
    if (lower === 'ctrl' || lower === 'control') {
      if (!modifiers.includes('Ctrl')) modifiers.push('Ctrl')
    } else if (lower === 'alt' || lower === 'option') {
      if (!modifiers.includes('Alt')) modifiers.push('Alt')
    } else if (lower === 'shift') {
      if (!modifiers.includes('Shift')) modifiers.push('Shift')
    } else {
      key = part.toUpperCase()
    }
  })

  if (!modifiers.length || !/^[A-Z0-9]$/.test(key)) return value.trim()
  return [...modifiers, key].join('+')
}

function shortcutFromKeyboardEvent(event) {
  const key = event.key.length === 1 ? event.key.toUpperCase() : ''
  if (!/^[A-Z0-9]$/.test(key)) return ''

  const modifiers = MODIFIER_LABELS
    .filter(([flag]) => event[flag])
    .map(([, label]) => label)

  if (!modifiers.length) return ''
  return [...modifiers, key].join('+')
}

function getGlobalInvoke() {
  return window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke
}

function hasTauriRuntime() {
  return Boolean(window.__TAURI_INTERNALS__ || getGlobalInvoke())
}

async function invokeCommand(command, args) {
  if (!hasTauriRuntime()) {
    throw new Error('当前不是 Tauri 桌面运行环境，请从桌面应用打开。')
  }

  const globalInvoke = getGlobalInvoke()
  if (globalInvoke) return globalInvoke(command, args)

  return tauriInvoke(command, args)
}

async function listenCommand(event, handler) {
  if (!hasTauriRuntime()) return null
  return tauriListen(event, handler)
}

function pulse(element) {
  if (!element) return
  element.classList.remove('pulse')
  window.requestAnimationFrame(() => element.classList.add('pulse'))
}

function render(payload) {
  if (!payload) return
  if (sourceEl) sourceEl.textContent = payload.source
  if (manualInput && payload.source && document.activeElement !== manualInput) {
    manualInput.value = payload.source
    updateInputCount()
  }
  if (targetEl) targetEl.textContent = payload.target
  if (providerEl && payload.provider) providerEl.textContent = payload.provider
  if (shortcutBadgeEl && payload.shortcut) shortcutBadgeEl.textContent = payload.shortcut.replaceAll('+', ' + ')
  if (languageResultEl) {
    languageResultEl.textContent = `${payload.source_lang || '-'} -> ${payload.target_lang || '-'}`
  }
  if (targetEl) targetEl.classList.toggle('loading', Boolean(payload.pending))
  if (payload.pending) {
    setStatus('翻译中...')
    return
  }
  setTranslationBusy(false)
  setStatus(payload.latency ? `${payload.latency} ms` : 'translated')
}

async function copyTargetText() {
  const text = targetEl?.textContent?.trim() || ''
  if (!text || text === '等待翻译' || text === '输入内容后自动翻译。') {
    setStatus('nothing to copy')
    return
  }

  try {
    await invokeCommand('copy_text', { text })
    setStatus('copied')
    if (copyTargetButton) {
      copyTargetButton.textContent = '已复制'
      window.setTimeout(() => {
        copyTargetButton.textContent = '复制'
      }, 1200)
    }
  } catch (error) {
    setStatus('copy failed')
    if (copyTargetButton) copyTargetButton.textContent = '复制失败'
    window.setTimeout(() => {
      if (copyTargetButton) copyTargetButton.textContent = '复制'
    }, 1200)
  }
}

function updateInputCount() {
  if (!manualInput || !inputCountEl) return
  inputCountEl.textContent = `${manualInput.value.length} / 5000`
}

function renderAutoTranslateState() {
  if (!autoTranslateToggle) return
  autoTranslateToggle.classList.toggle('disabled', !autoTranslateEnabled)
  autoTranslateToggle.textContent = autoTranslateEnabled ? '● 自动翻译已开启' : '○ 自动翻译已关闭'
}

function setAutoTranslateEnabled(enabled) {
  autoTranslateEnabled = enabled
  renderAutoTranslateState()

  if (!autoTranslateEnabled) {
    cancelAutoTranslate()
    queuedAutoTranslate = false
    queuedAutoSignature = ''
    setStatus('auto translate off')
    return
  }

  setStatus('auto translate on')
  scheduleAutoTranslate(getAutoTranslateDelay())
}

function getAutoTranslateDelay(inputType = '') {
  if (inputType === 'insertFromPaste') return AUTO_TRANSLATE_PASTE_DELAY_MS

  const textLength = manualInput?.value.trim().length || 0
  if (textLength <= 20) return AUTO_TRANSLATE_SHORT_DELAY_MS
  if (textLength <= 120) return AUTO_TRANSLATE_DEFAULT_DELAY_MS
  return AUTO_TRANSLATE_LONG_DELAY_MS
}

function updateLangpairMeta() {
  const sourceLang = sourceLangEl?.value || 'auto'
  const targetLang = targetLangEl?.value || 'zh-CN'
  if (providerLangpairEl) providerLangpairEl.textContent = `${sourceLang} -> ${targetLang}`
}

function activeProvider() {
  return (
    availableProviders().find((provider) => provider.id === providerState.active_provider_id) ||
    availableProviders()[0] ||
    providerState.providers[0]
  )
}

function providerById(providerId) {
  return providerState.providers.find((provider) => provider.id === providerId)
}

function providerDisplayName(provider) {
  if (!provider) return ''
  if (provider.protocol === 'mymemory') return provider.name
  return provider.api_key_configured ? provider.name : `${provider.name}（未配置）`
}

function availableProviders() {
  return providerState.providers
}

function renderProviderOptions() {
  const activeOptions = availableProviders()
    .map((provider) => `<option value="${provider.id}">${providerDisplayName(provider)}</option>`)
    .join('')
  const configOptions = [
    ...providerState.providers.map(
      (provider) => `<option value="${provider.id}">${providerDisplayName(provider)}</option>`,
    ),
    `<option value="${CUSTOM_PROVIDER_VALUE}">添加自定义 OpenAI 兼容服务</option>`,
  ].join('')

  if (activeProviderInput) {
    activeProviderInput.innerHTML = activeOptions
    activeProviderInput.value = availableProviders().some(
      (provider) => provider.id === providerState.active_provider_id,
    )
      ? providerState.active_provider_id
      : DEFAULT_PROVIDER_ID
    refreshCustomSelect(activeProviderInput)
  }

  if (configProviderInput) {
    configProviderInput.innerHTML = configOptions
    configProviderInput.value = providerState.active_provider_id
    refreshCustomSelect(configProviderInput)
  }
}

function updateProviderView() {
  renderProviderOptions()
  const provider = activeProvider()
  if (!provider) return

  if (providerEl) providerEl.textContent = provider.name
  if (providerAiStateEl) {
    providerAiStateEl.textContent =
      provider.protocol === 'mymemory'
        ? '公共接口'
        : provider.api_key_configured
          ? '已配置'
          : '未配置'
  }
}

function applyProviderDefaults() {
  const preset = PROVIDER_PRESETS[configProviderInput?.value] || PROVIDER_PRESETS.deepseek
  if (providerNameInput) providerNameInput.value = preset.name
  if (providerProtocolInput) providerProtocolInput.value = preset.protocol
  if (aiBaseUrlInput) aiBaseUrlInput.value = preset.base_url
  if (aiModelInput) aiModelInput.value = preset.model
  if (providerAgentInput) providerAgentInput.value = ''
}

function toggleCustomProviderFields(isCustom) {
  customProviderFields.forEach((field) => {
    field.hidden = !isCustom
  })
  if (providerConfigNoteEl) {
    providerConfigNoteEl.textContent = isCustom
      ? '自定义服务使用 OpenAI 兼容 /chat/completions 接口，Agent 内置；API Key 保存到系统凭据。'
      : 'DeepSeek 使用官方 OpenAI 兼容接口，模型和 Agent 已内置。API Key 保存到系统凭据。'
  }
}

function createCustomProviderId(name, model) {
  const source = `${name || 'custom'}-${model || Date.now()}`
  const slug = source
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '')
    .slice(0, 36)
  return `custom-${slug || Date.now()}`
}

function loadCustomProviderForm() {
  if (configProviderInput) configProviderInput.value = CUSTOM_PROVIDER_VALUE
  if (providerNameInput) providerNameInput.value = ''
  if (providerProtocolInput) providerProtocolInput.value = 'openai'
  if (aiBaseUrlInput) aiBaseUrlInput.value = ''
  if (aiModelInput) aiModelInput.value = ''
  if (providerAgentInput) providerAgentInput.value = ''
  if (aiKeyInput) {
    aiKeyInput.value = ''
    aiKeyInput.placeholder = '填写该服务的 API Key'
  }
  toggleCustomProviderFields(true)
  refreshCustomSelect(configProviderInput)
}

function loadProviderIntoForm(providerId = providerState.active_provider_id) {
  if (providerId === CUSTOM_PROVIDER_VALUE) {
    loadCustomProviderForm()
    return
  }

  const provider = providerById(providerId)
  if (!provider) return

  if (configProviderInput) configProviderInput.value = provider.id
  const preset = PROVIDER_PRESETS[provider.id]
  const isCustom = !preset && provider.protocol !== 'mymemory'
  if (providerNameInput) providerNameInput.value = preset?.name || provider.name
  if (providerProtocolInput) providerProtocolInput.value = preset?.protocol || provider.protocol
  if (aiBaseUrlInput) aiBaseUrlInput.value = preset?.base_url ?? provider.base_url ?? ''
  if (aiModelInput) aiModelInput.value = preset?.model ?? provider.model ?? ''
  if (providerAgentInput) providerAgentInput.value = provider.agent_prompt || ''
  if (aiKeyInput) {
    aiKeyInput.value = ''
    aiKeyInput.placeholder = provider.api_key_configured
      ? 'API Key 已保存，留空则继续保留'
      : provider.id === 'deepseek'
        ? 'DeepSeek 只需要填写 API Key'
        : '填写该服务的 API Key'
  }
  toggleCustomProviderFields(isCustom)
  refreshCustomSelect(configProviderInput)
}

function cancelAutoTranslate() {
  if (!autoTranslateTimer) return
  window.clearTimeout(autoTranslateTimer)
  autoTranslateTimer = null
}

function scheduleAutoTranslate(delay = getAutoTranslateDelay()) {
  cancelAutoTranslate()

  if (!autoTranslateEnabled) {
    queuedAutoTranslate = false
    queuedAutoSignature = ''
    setStatus('auto translate off')
    return
  }

  const text = manualInput?.value.trim()
  if (!text) {
    queuedAutoTranslate = false
    queuedAutoSignature = ''
    translationRequestId += 1
    if (targetEl) targetEl.textContent = '输入内容后自动翻译。'
    if (languageResultEl) languageResultEl.textContent = '等待翻译'
    setStatus('ready')
    return
  }

  const signature = getTranslationSignature(text)
  if (signature === lastTranslatedSignature) {
    setStatus('translated')
    return
  }

  if (isTranslating) {
    if (signature === activeTranslationSignature || signature === queuedAutoSignature) {
      setStatus('翻译中...')
      return
    }
    queuedAutoTranslate = true
    queuedAutoSignature = signature
    translationRequestId += 1
    setStatus('等待上一条翻译完成...')
    return
  }

  setStatus('waiting...')
  autoTranslateTimer = window.setTimeout(() => {
    translateManual({ mode: 'auto' })
  }, delay)
}

async function refreshStatus() {
  const status = await invokeCommand('get_app_status')
  if (shortcutEl) shortcutEl.textContent = status.shortcut
  if (selectionShortcutEl) selectionShortcutEl.textContent = status.shortcut
  if (shortcutBadgeEl) shortcutBadgeEl.textContent = status.shortcut.replaceAll('+', ' + ')
  if (providerEl) providerEl.textContent = status.provider
  if (autostartInput) autostartInput.checked = status.autostart_enabled
  if (settingsAutostartEl) settingsAutostartEl.textContent = status.autostart_enabled ? '已开启' : '未开启'
  if (settingsAutostartToggle) settingsAutostartToggle.checked = status.autostart_enabled
  if (settingsShortcutInput) settingsShortcutInput.value = status.shortcut
  if (settingsMainShortcutInput) settingsMainShortcutInput.value = status.main_shortcut || 'Ctrl+D'
  if (settingsStartupModeInput) settingsStartupModeInput.value = status.startup_mode || 'main'
  if (status.theme) applyTheme(status.theme, { persist: false })
  if (settingsThemeInput) settingsThemeInput.value = status.theme || document.documentElement.dataset.theme || 'dark'
  refreshCustomSelect(settingsStartupModeInput)
  refreshCustomSelect(settingsThemeInput)
  setStatus('ready')
}

async function saveRuntimeSettings() {
  setStatus('saving settings...')
  const originalButtonText = saveSettingsButton?.textContent || '保存设置'
  if (saveSettingsButton) {
    saveSettingsButton.disabled = true
    saveSettingsButton.textContent = '保存中...'
  }
  let autostartWarning = ''
  try {
    const shortcut = normalizeShortcutInput(settingsShortcutInput?.value || 'Alt+D')
    const mainShortcut = normalizeShortcutInput(settingsMainShortcutInput?.value || 'Ctrl+D')
    if (settingsShortcutInput) settingsShortcutInput.value = shortcut
    if (settingsMainShortcutInput) settingsMainShortcutInput.value = mainShortcut
    const settings = {
      shortcut,
      main_shortcut: mainShortcut,
      startup_mode: settingsStartupModeInput?.value || 'main',
      theme: settingsThemeInput?.value || 'dark',
    }
    const saved = await invokeCommand('set_runtime_settings', { settings })
    if (settingsShortcutInput) settingsShortcutInput.value = saved.shortcut
    if (settingsMainShortcutInput) settingsMainShortcutInput.value = saved.main_shortcut
    if (settingsStartupModeInput) settingsStartupModeInput.value = saved.startup_mode
    if (settingsThemeInput) settingsThemeInput.value = saved.theme || settingsThemeInput.value
    refreshCustomSelect(settingsStartupModeInput)
    refreshCustomSelect(settingsThemeInput)

    if (settingsAutostartToggle) {
      try {
        const status = await invokeCommand('set_autostart', { enabled: settingsAutostartToggle.checked })
        if (autostartInput) autostartInput.checked = status.autostart_enabled
        settingsAutostartToggle.checked = status.autostart_enabled
        if (settingsAutostartEl) settingsAutostartEl.textContent = status.autostart_enabled ? '已开启' : '未开启'
      } catch (error) {
        const currentStatus = await invokeCommand('get_app_status').catch(() => null)
        if (currentStatus) {
          if (autostartInput) autostartInput.checked = currentStatus.autostart_enabled
          settingsAutostartToggle.checked = currentStatus.autostart_enabled
          if (settingsAutostartEl) settingsAutostartEl.textContent = currentStatus.autostart_enabled ? '已开启' : '未开启'
        }
        autostartWarning = `开机自启未保存：${String(error)}`
      }
    }

    if (shortcutEl) shortcutEl.textContent = saved.shortcut
    if (selectionShortcutEl) selectionShortcutEl.textContent = saved.shortcut
    if (settingsThemeInput) applyTheme(settingsThemeInput.value)
    if (settingsNoteEl) {
      settingsNoteEl.textContent = autostartWarning || '设置已保存。快捷键已立即生效。'
    }
    if (saveSettingsButton) saveSettingsButton.textContent = autostartWarning ? '已保存' : '已保存'
    showToast(autostartWarning || '设置已保存，快捷键已立即生效')
    setStatus(autostartWarning ? 'settings saved with warning' : 'settings saved')
  } catch (error) {
    const message = String(error)
    if (settingsNoteEl) settingsNoteEl.textContent = message
    showToast(`保存失败：${message}`, 'error')
    setStatus('settings failed')
  } finally {
    if (saveSettingsButton) {
      window.setTimeout(() => {
        saveSettingsButton.disabled = false
        saveSettingsButton.textContent = originalButtonText
      }, 900)
    }
  }
}

async function refreshProviderState() {
  providerState = await invokeCommand('get_provider_state')
  updateProviderView()
  loadProviderIntoForm(providerState.active_provider_id)
}

async function setActiveProvider(providerId) {
  setStatus('switching provider...')
  providerState = await invokeCommand('set_active_provider', { providerId })
  updateProviderView()
  loadProviderIntoForm(providerState.active_provider_id)
  setStatus('provider selected')
  scheduleAutoTranslate()
}

async function saveProviderConfig() {
  if (providerConfigNoteEl) providerConfigNoteEl.textContent = '正在保存翻译服务配置...'
  const originalButtonText = saveAiConfigButton?.textContent || '保存并使用'
  if (saveAiConfigButton) {
    saveAiConfigButton.disabled = true
    saveAiConfigButton.textContent = '保存中...'
  }
  const selectedConfigValue = configProviderInput?.value || 'deepseek'
  const isCustomProvider = selectedConfigValue === CUSTOM_PROVIDER_VALUE
  const selectedProvider = isCustomProvider ? null : providerById(selectedConfigValue)
  const customName = providerNameInput?.value?.trim() || ''
  const customModel = aiModelInput?.value?.trim() || ''
  const providerId = isCustomProvider
    ? createCustomProviderId(customName, customModel)
    : selectedConfigValue
  const preset = PROVIDER_PRESETS[providerId]
  const apiKey = aiKeyInput?.value?.trim() || ''
  const protocol = isCustomProvider ? 'openai' : preset?.protocol || selectedProvider?.protocol || 'openai'
  const baseUrl = isCustomProvider
    ? aiBaseUrlInput?.value?.trim() || ''
    : preset?.base_url ?? selectedProvider?.base_url ?? ''
  const model = isCustomProvider ? customModel : preset?.model ?? selectedProvider?.model ?? ''

  if (protocol !== 'mymemory' && !apiKey && !selectedProvider?.api_key_configured) {
    setStatus('api key required')
    if (providerConfigNoteEl) providerConfigNoteEl.textContent = '请先填写 API Key，再保存并使用该 AI 翻译服务。'
    if (targetEl) targetEl.textContent = '请先填写 API Key，再保存并使用该 AI 翻译服务。'
    showToast('请先填写 API Key', 'error')
    if (saveAiConfigButton) {
      saveAiConfigButton.disabled = false
      saveAiConfigButton.textContent = originalButtonText
    }
    return
  }

  if (isCustomProvider && (!customName || !baseUrl || !model)) {
    setStatus('provider fields required')
    if (providerConfigNoteEl) providerConfigNoteEl.textContent = '自定义服务需要填写名称、Base URL、Model 和 API Key。'
    if (targetEl) targetEl.textContent = '自定义服务需要填写名称、Base URL、Model 和 API Key。'
    showToast('请补全自定义服务配置', 'error')
    if (saveAiConfigButton) {
      saveAiConfigButton.disabled = false
      saveAiConfigButton.textContent = originalButtonText
    }
    return
  }

  const config = {
    id: providerId,
    name: isCustomProvider ? customName : preset?.name || selectedProvider?.name || '自定义服务',
    protocol,
    base_url: baseUrl,
    model,
    agent_prompt: providerAgentInput?.value || selectedProvider?.agent_prompt || '',
    api_key: apiKey || null,
  }

  setStatus('saving provider...')
  try {
    providerState = await invokeCommand('save_provider_config', { config })
    providerState = await invokeCommand('set_active_provider', { providerId: config.id })
    if (aiKeyInput) aiKeyInput.value = ''
    updateProviderView()
    loadProviderIntoForm(config.id)
    if (providerConfigNoteEl) providerConfigNoteEl.textContent = `${config.name} 已保存并设为当前翻译服务。`
    if (saveAiConfigButton) saveAiConfigButton.textContent = '已保存'
    showToast(`${config.name} 已保存并启用`)
    setStatus('provider saved')
    scheduleAutoTranslate()
  } catch (error) {
    const message = String(error)
    setStatus('provider failed')
    if (providerConfigNoteEl) providerConfigNoteEl.textContent = `保存失败：${message}`
    if (targetEl) targetEl.textContent = message
    showToast(`保存失败：${message}`, 'error')
  } finally {
    if (saveAiConfigButton) {
      window.setTimeout(() => {
        saveAiConfigButton.disabled = false
        saveAiConfigButton.textContent = originalButtonText
      }, 900)
    }
  }
}

async function translateManual(options = {}) {
  cancelAutoTranslate()

  const text = manualInput?.value.trim()
  if (!text) {
    setStatus('empty input')
    return
  }

  const sourceLang = sourceLangEl?.value || 'auto'
  const targetLang = targetLangEl?.value || 'zh-CN'
  const signature = getTranslationSignature(text)

  if (isTranslating) {
    if (signature === activeTranslationSignature || signature === queuedAutoSignature) {
      setStatus('翻译中...')
      return
    }
    queuedAutoTranslate = true
    queuedAutoSignature = signature
    translationRequestId += 1
    setStatus('等待上一条翻译完成...')
    return
  }

  const requestId = ++translationRequestId
  activeTranslationRequestId = requestId
  isTranslating = true
  activeTranslationSignature = signature
  if (options.mode === 'auto' && signature === lastTranslatedSignature) {
    setStatus('translated')
    isTranslating = false
    activeTranslationRequestId = 0
    activeTranslationSignature = ''
    return
  }
  if (signature === queuedAutoSignature) queuedAutoSignature = ''
  setStatus(options.mode === 'auto' ? '自动翻译中...' : '翻译中...')
  setTranslationBusy(true, options.mode === 'auto' ? '正在自动翻译...' : '正在翻译...')
  try {
    const payload = await invokeCommand('translate_manual', {
      text,
      sourceLang,
      targetLang,
    })
    if (requestId !== translationRequestId) return
    lastTranslatedSignature = signature
    render(payload)
  } catch (error) {
    if (requestId !== translationRequestId) return
    setStatus('request failed')
    setTranslationBusy(false)
    if (targetEl) targetEl.textContent = String(error)
  } finally {
    if (activeTranslationRequestId === requestId) {
      isTranslating = false
      activeTranslationRequestId = 0
      activeTranslationSignature = ''
      setTranslationBusy(false)
      if (queuedAutoTranslate) {
        queuedAutoTranslate = false
        scheduleAutoTranslate(QUEUED_TRANSLATE_DELAY_MS)
      }
    }
  }
}

async function simulateSelection() {
  setStatus('translating...')
  try {
    await invokeCommand('simulate_translation')
    setStatus('selection popup opened')
  } catch (error) {
    setStatus('request failed')
    showToast(String(error), 'error')
  }
}

function activateNav(action, options = {}) {
  const nextAction = viewCopy[action] ? action : 'translate'
  if (nextAction === 'provider') aiConfigOpen = true
  if (nextAction === 'translate' && !options.keepConfig) aiConfigOpen = false
  if (nextAction === 'selection' || nextAction === 'settings') aiConfigOpen = false

  navButtons.forEach((button) => {
    button.classList.toggle('active', button.dataset.navAction === nextAction)
  })

  viewSections.forEach((section) => {
    const allowed = section.dataset.viewSection.split(' ').includes(nextAction)
    const shouldHideAi = section.id === 'ai-config' && !aiConfigOpen
    section.hidden = !allowed || shouldHideAi
  })

  if (viewTitleEl) viewTitleEl.textContent = viewCopy[nextAction].title
  if (viewSubtitleEl) viewSubtitleEl.textContent = viewCopy[nextAction].subtitle
  updateLangpairMeta()

  if (nextAction === 'translate') {
    manualInput?.focus()
    setStatus('translate ready')
  }

  if (nextAction === 'provider') {
    loadProviderIntoForm(providerState.active_provider_id)
    setStatus('provider settings')
  }

  if (nextAction === 'selection') {
    pulse(simulateSelectionViewButton)
    setStatus('use Alt + D')
  }

  if (nextAction === 'settings') {
    aiConfigOpen = false
    pulse(systemPanel)
    setStatus('settings')
  }
}

listenCommand('translation-ready', (event) => {
  if (!isPopupPage && event.payload?.surface === 'popup') return
  render(event.payload)
}).catch(() => setStatus('runtime unavailable'))

async function refreshPopupPayload() {
  if (!isPopupPage) return
  try {
    const payload = await invokeCommand('get_popup_payload')
    if (payload) render(payload)
  } catch {
    // The event listener remains the primary path; polling is a popup-only fallback.
  }
}

if (translateButton) translateButton.addEventListener('click', translateManual)
if (copyTargetButton) copyTargetButton.addEventListener('click', copyTargetText)
if (simulateButton) simulateButton.addEventListener('click', simulateSelection)
if (simulateSelectionViewButton) simulateSelectionViewButton.addEventListener('click', simulateSelection)
if (refreshButton) refreshButton.addEventListener('click', refreshStatus)

windowActionButtons.forEach((button) => {
  button.addEventListener('click', async (event) => {
    event.preventDefault()
    event.stopPropagation()
    const action = button.dataset.windowAction
    try {
      await invokeCommand('window_action', { action })
    } catch (error) {
      setStatus(`window action failed: ${String(error)}`)
    }
  })
})

document.querySelectorAll('[data-tauri-drag-region]').forEach((region) => {
  region.addEventListener('pointerdown', async (event) => {
    if (event.button !== 0) return
    if (event.target.closest('button, input, select, textarea, a')) return

    try {
      await getCurrentWindow().startDragging()
    } catch {
      // Tauri's native drag region still works when this fallback is unavailable.
    }
  })
})

navButtons.forEach((button) => {
  button.addEventListener('click', () => activateNav(button.dataset.navAction))
})

if (manualInput) {
  manualInput.addEventListener('input', (event) => {
    updateInputCount()
    scheduleAutoTranslate(getAutoTranslateDelay(event.inputType))
  })
  manualInput.addEventListener('pointerleave', () => {
    scheduleAutoTranslate(AUTO_TRANSLATE_LEAVE_DELAY_MS)
  })
  manualInput.addEventListener('blur', () => {
    scheduleAutoTranslate(AUTO_TRANSLATE_LEAVE_DELAY_MS)
  })
  manualInput.addEventListener('keydown', (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
      event.preventDefault()
      translateManual()
    }
  })
  updateInputCount()
}

if (autoTranslateToggle) {
  autoTranslateToggle.addEventListener('click', () => {
    setAutoTranslateEnabled(!autoTranslateEnabled)
  })
  renderAutoTranslateState()
}

if (swapLangButton) {
  swapLangButton.addEventListener('click', () => {
    if (!sourceLangEl || !targetLangEl) return
    if (sourceLangEl.value === 'auto') sourceLangEl.value = 'en'
    const source = sourceLangEl.value
    sourceLangEl.value = targetLangEl.value
    targetLangEl.value = source
    refreshCustomSelect(sourceLangEl)
    refreshCustomSelect(targetLangEl)
    updateLangpairMeta()
    setStatus('language swapped')
  })
}

if (sourceLangEl) sourceLangEl.addEventListener('change', updateLangpairMeta)
if (targetLangEl) targetLangEl.addEventListener('change', updateLangpairMeta)

if (sourceLangEl) {
  sourceLangEl.addEventListener('change', scheduleAutoTranslate)
}

if (targetLangEl) {
  targetLangEl.addEventListener('change', scheduleAutoTranslate)
}

if (clearInputButton) {
  clearInputButton.addEventListener('click', () => {
    if (manualInput) manualInput.value = ''
    cancelAutoTranslate()
    translationRequestId += 1
    queuedAutoTranslate = false
    queuedAutoSignature = ''
    isTranslating = false
    activeTranslationRequestId = 0
    activeTranslationSignature = ''
    lastTranslatedSignature = ''
    if (targetEl) targetEl.textContent = '输入内容后自动翻译。'
    if (languageResultEl) languageResultEl.textContent = '等待翻译'
    updateInputCount()
    setStatus('cleared')
    manualInput?.focus()
  })
}

if (autostartInput) {
  autostartInput.addEventListener('change', async () => {
    setStatus('updating...')
    try {
      const status = await invokeCommand('set_autostart', { enabled: autostartInput.checked })
      autostartInput.checked = status.autostart_enabled
      setStatus(status.autostart_enabled ? 'autostart on' : 'autostart off')
    } catch (error) {
      autostartInput.checked = !autostartInput.checked
      setStatus('autostart failed')
    }
  })
}

if (saveSettingsButton) {
  saveSettingsButton.addEventListener('click', saveRuntimeSettings)
}

if (settingsShortcutInput) {
  settingsShortcutInput.addEventListener('keydown', (event) => {
    const shortcut = shortcutFromKeyboardEvent(event)
    if (!shortcut) return

    event.preventDefault()
    settingsShortcutInput.value = shortcut
    if (settingsNoteEl) settingsNoteEl.textContent = '按保存后快捷键立即生效。'
  })

  settingsShortcutInput.addEventListener('blur', () => {
    settingsShortcutInput.value = normalizeShortcutInput(settingsShortcutInput.value)
  })
}

if (settingsMainShortcutInput) {
  settingsMainShortcutInput.addEventListener('keydown', (event) => {
    const shortcut = shortcutFromKeyboardEvent(event)
    if (!shortcut) return

    event.preventDefault()
    settingsMainShortcutInput.value = shortcut
    if (settingsNoteEl) settingsNoteEl.textContent = '按保存后打开主界面快捷键立即生效。'
  })

  settingsMainShortcutInput.addEventListener('blur', () => {
    settingsMainShortcutInput.value = normalizeShortcutInput(settingsMainShortcutInput.value)
  })
}

if (configureAiButton) {
  configureAiButton.addEventListener('click', () => {
    aiConfigOpen = true
    activateNav('translate', { keepConfig: true })
    loadProviderIntoForm(providerState.active_provider_id)
  })
}

if (closeAiButton) {
  closeAiButton.addEventListener('click', () => {
    aiConfigOpen = false
    activateNav('translate')
    setStatus('ready')
  })
}

if (saveAiConfigButton) {
  saveAiConfigButton.addEventListener('click', saveProviderConfig)
}

if (activeProviderInput) {
  activeProviderInput.addEventListener('change', () => {
    setActiveProvider(activeProviderInput.value).catch((error) => {
      setStatus('provider failed')
      if (targetEl) targetEl.textContent = String(error)
    })
  })
}

if (configProviderInput) {
  configProviderInput.addEventListener('change', () => {
    loadProviderIntoForm(configProviderInput.value)
  })
}

if (settingsThemeInput) {
  settingsThemeInput.addEventListener('change', () => applyTheme(settingsThemeInput.value))
}

document.addEventListener('click', (event) => {
  if (!event.target.closest('.custom-select')) closeCustomSelects()
})

document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') closeCustomSelects()
})

enhanceCustomSelects()
applyTheme(window.localStorage.getItem(THEME_STORAGE_KEY) || 'dark')

async function bootstrapMainWindow() {
  try {
    await refreshStatus()
    await refreshProviderState()
    updateLangpairMeta()
    activateNav('translate')
  } catch (error) {
    const message = `启动配置读取失败：${String(error)}`
    setStatus('config load failed')
    if (targetEl) targetEl.textContent = message
    showToast(message, 'error')
  } finally {
    document.body.classList.add('app-ready')
    try {
      const currentWindow = getCurrentWindow()
      await currentWindow.show()
      await currentWindow.setFocus()
    } catch {
      // The window may already be visible when running outside a packaged desktop window.
    }
  }
}

if (isPopupPage) {
  refreshPopupPayload()
  window.setInterval(refreshPopupPayload, 250)
  document.body.classList.add('app-ready')
} else {
  bootstrapMainWindow()
}
