<script setup lang="ts">
import main from '@/assets/images/main.png'

const highlights = [
  {
    title: 'AI Session Sync',
    description: 'Stay aligned with Claude Code, Copilot, and Codex, and surface thinking, tool use, completion, and attention states through a desktop companion.',
  },
  {
    title: 'Live2D Desktop Presence',
    description: 'Ship with the built-in Hiyori model in a transparent always-on-top window that behaves like a navigator, not another buried app.',
  },
  {
    title: 'Configurable Workflow',
    description: 'Adjust name, language, window size, motion bindings, model directory, and launch behavior to fit your own setup.',
  },
]

const moments = [
  {
    state: 'Thinking',
    detail: 'When an agent enters a thinking phase, Copiwaifu reflects it on the desktop so you do not have to keep checking the terminal for progress.',
  },
  {
    state: 'Tool Use',
    detail: 'While commands are running or tools are waiting on results, Copiwaifu keeps the current rhythm visible in your peripheral vision.',
  },
  {
    state: 'Needs Attention',
    detail: 'When approval is needed, an error happens, or a task finishes, the signal moves to the desktop immediately so key interactions are harder to miss.',
  },
]

const downloads = [
  {
    title: 'Latest Release',
    description: 'Open GitHub Releases and grab the installer for your platform.',
    href: 'https://github.com/Panzer-Jack/Copiwaifu/releases/latest',
    label: 'Download Copiwaifu',
  },
  {
    title: 'Source Repository',
    description: 'Browse the codebase, release notes, and ongoing development.',
    href: 'https://github.com/Panzer-Jack/Copiwaifu',
    label: 'View GitHub',
  },
]
</script>

<template>
  <div class="landing-shell">
    <div class="landing-shell__grid" />
    <div class="landing-shell__glow landing-shell__glow--left" />
    <div class="landing-shell__glow landing-shell__glow--right" />

    <main class="relative z-1 mx-auto max-w-1200px px-20px pb-32px pt-20px text-text-primary md:px-32px md:pb-48px md:pt-28px">
      <header class="mb-24px flex items-center justify-between rounded-24px border border-white/10 bg-white/6 px-18px py-14px backdrop-blur-14px md:mb-32px md:px-24px">
        <div>
          <p class="text-12px uppercase tracking-4px text-[#f3c98b] md:text-13px">
            Copiwaifu
          </p>
          <p class="mt-6px text-13px text-text-secondary md:text-14px">
            Your Live2D AI Navigator for every coding mission.
          </p>
        </div>
        <a
          href="#download"
          class="inline-flex items-center gap-8px rounded-full border border-[#f3c98b]/40 bg-[#f3c98b]/10 px-16px py-10px text-13px font-600 text-[#ffe7bf] no-underline transition-all duration-300 hover:border-[#f3c98b]/70 hover:bg-[#f3c98b]/18"
        >
          Download
        </a>
      </header>

      <section class="grid gap-24px md:grid-cols-[minmax(0,1.05fr)_minmax(360px,0.95fr)] md:items-center md:gap-28px">
        <div class="rounded-[32px] border border-white/10 bg-[#120f18]/78 p-22px shadow-[0_18px_80px_rgba(0,0,0,0.35)] backdrop-blur-20px md:p-32px">
          <div class="inline-flex items-center gap-10px rounded-full border border-[#ff9d4d]/30 bg-[#ff9d4d]/8 px-14px py-8px text-12px text-[#ffd3ad]">
            <span class="h-7px w-7px rounded-full bg-[#ffb86b]" />
            Live2D companion for Claude Code, Copilot and Codex
          </div>

          <h1 class="mt-18px max-w-640px text-36px leading-[1.05] font-700 tracking-[-1.6px] text-white md:text-58px">
            Bring your AI coding sessions out of the terminal and onto the desktop.
          </h1>

          <p class="mt-16px max-w-620px text-15px leading-26px text-text-secondary md:text-18px md:leading-30px">
            Copiwaifu is a desktop companion built with Tauri 2 and Vue 3, powered by the Live2D character Hiyori.
            It syncs with your AI coding agents in real time and turns thinking, tool use, completion, and attention states into visible desktop feedback.
          </p>

          <div class="mt-22px flex flex-wrap gap-12px">
            <a
              href="https://github.com/Panzer-Jack/Copiwaifu/releases/latest"
              target="_blank"
              rel="noreferrer"
              class="inline-flex items-center justify-center rounded-full bg-[#f3c98b] px-20px py-12px text-14px font-700 text-[#1d140d] no-underline transition-all duration-300 hover:translate-y-[-1px] hover:bg-[#ffd8a6]"
            >
              Download the latest release
            </a>
            <a
              href="https://github.com/Panzer-Jack/Copiwaifu"
              target="_blank"
              rel="noreferrer"
              class="inline-flex items-center justify-center rounded-full border border-white/14 bg-white/5 px-20px py-12px text-14px font-600 text-white no-underline transition-all duration-300 hover:border-white/26 hover:bg-white/10"
            >
              View source
            </a>
          </div>

          <div class="mt-24px grid gap-12px md:grid-cols-3">
            <div class="rounded-22px border border-white/10 bg-black/12 p-16px">
              <p class="text-12px uppercase tracking-3px text-[#ffcf93]">
                Sync
              </p>
              <p class="mt-10px text-15px leading-24px text-white">
                Mirror agent state and tool activity
              </p>
            </div>
            <div class="rounded-22px border border-white/10 bg-black/12 p-16px">
              <p class="text-12px uppercase tracking-3px text-[#ffcf93]">
                Presence
              </p>
              <p class="mt-10px text-15px leading-24px text-white">
                Stay visible without taking over your workspace
              </p>
            </div>
            <div class="rounded-22px border border-white/10 bg-black/12 p-16px">
              <p class="text-12px uppercase tracking-3px text-[#ffcf93]">
                Custom
              </p>
              <p class="mt-10px text-15px leading-24px text-white">
                Tune model, language, size, and motion behavior
              </p>
            </div>
          </div>
        </div>

        <div class="hero-visual rounded-[32px] border border-white/10 bg-[#0f0b14]/84 p-16px shadow-[0_28px_90px_rgba(0,0,0,0.42)] backdrop-blur-18px md:p-18px">
          <div class="hero-visual__frame rounded-[26px]">
            <img
              :src="main"
              alt="Copiwaifu visual"
              class="block h-100% w-100% object-cover object-center"
            >
          </div>

          <div class="hero-visual__panel">
            <p class="text-11px uppercase tracking-3px text-[#ffd59d]">
              Runtime Snapshot
            </p>
            <h2 class="mt-8px text-20px font-700 text-white">
              More than a desktop pet, it is a status surface.
            </h2>
            <p class="mt-10px text-14px leading-24px text-text-secondary">
              When approvals are pending, tools are still running, or a task has completed, Copiwaifu raises the signal to desktop level so you can react faster.
            </p>

            <div class="mt-16px flex flex-wrap gap-10px">
              <span class="hero-chip">
                Claude Code
              </span>
              <span class="hero-chip">
                Copilot
              </span>
              <span class="hero-chip">
                Codex
              </span>
              <span class="hero-chip">
                Live2D
              </span>
            </div>
          </div>
        </div>
      </section>

      <section
        id="features"
        class="mt-24px rounded-[32px] border border-white/10 bg-[#120f18]/74 p-22px backdrop-blur-18px md:mt-28px md:p-30px"
      >
        <div class="flex flex-col gap-12px md:flex-row md:items-end md:justify-between">
          <div>
            <p class="text-12px uppercase tracking-4px text-[#f3c98b]">
              What it does
            </p>
            <h2 class="mt-10px text-28px font-700 text-white md:text-36px">
              What Copiwaifu does on a developer desktop
            </h2>
          </div>
          <p class="max-w-460px text-14px leading-24px text-text-secondary md:text-right">
            It is not a chat box and not another IDE sidebar. It gives an AI coding workflow a continuous, low-friction visual presence.
          </p>
        </div>

        <div class="mt-22px grid gap-14px md:grid-cols-3">
          <article
            v-for="item in highlights"
            :key="item.title"
            class="rounded-24px border border-white/10 bg-black/14 p-18px transition-transform duration-300 hover:translate-y-[-2px]"
          >
            <p class="text-12px uppercase tracking-3px text-[#ffd6a8]">
              Capability
            </p>
            <h3 class="mt-12px text-20px font-700 text-white">
              {{ item.title }}
            </h3>
            <p class="mt-12px text-14px leading-24px text-text-secondary">
              {{ item.description }}
            </p>
          </article>
        </div>
      </section>

      <section class="mt-24px grid gap-18px md:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)] md:mt-28px">
        <div class="rounded-[32px] border border-white/10 bg-[#120f18]/74 p-22px backdrop-blur-18px md:p-30px">
          <p class="text-12px uppercase tracking-4px text-[#f3c98b]">
            Workflow
          </p>
          <h2 class="mt-10px text-28px font-700 text-white md:text-34px">
            Built for long sessions with coding agents
          </h2>
          <p class="mt-14px text-14px leading-24px text-text-secondary">
            When your attention is split across the browser, design files, logs, and terminal panes, Copiwaifu keeps the most important moment of the AI session visible on the desktop.
          </p>
        </div>

        <div class="grid gap-12px">
          <article
            v-for="moment in moments"
            :key="moment.state"
            class="rounded-[28px] border border-white/10 bg-[#100c15]/80 p-18px backdrop-blur-16px"
          >
            <p class="text-12px uppercase tracking-3px text-[#ffcf93]">
              {{ moment.state }}
            </p>
            <p class="mt-10px text-14px leading-25px text-text-secondary md:text-15px">
              {{ moment.detail }}
            </p>
          </article>
        </div>
      </section>

      <section
        id="download"
        class="mt-24px rounded-[36px] border border-[#f3c98b]/22 bg-[linear-gradient(135deg,rgba(243,201,139,0.14),rgba(16,12,21,0.94)_34%,rgba(14,10,18,0.98))] p-22px shadow-[0_24px_100px_rgba(0,0,0,0.34)] md:mt-28px md:p-32px"
      >
        <div class="grid gap-18px md:grid-cols-[minmax(0,1fr)_minmax(320px,0.92fr)] md:items-start">
          <div>
            <p class="text-12px uppercase tracking-4px text-[#ffe0b4]">
              Download
            </p>
            <h2 class="mt-10px text-30px font-700 text-white md:text-42px">
              Download Copiwaifu and start syncing your AI desktop presence.
            </h2>
            <p class="mt-14px max-w-620px text-15px leading-26px text-[#d3c7df]">
              The app ships through Tauri builds. Open the latest release, choose the installer for your platform, and launch it. In-app updating is already wired in for future upgrades.
            </p>
          </div>

          <div class="rounded-[28px] border border-white/10 bg-black/18 p-18px backdrop-blur-14px">
            <p class="text-12px uppercase tracking-3px text-[#ffd49f]">
              Quick Start
            </p>
            <ol class="mt-12px list-decimal pl-20px text-14px leading-26px text-[#ece3f7]">
              <li>Open the latest release page.</li>
              <li>Pick the installer that matches your platform.</li>
              <li>Launch Copiwaifu and connect it to your AI coding workflow.</li>
            </ol>
          </div>
        </div>

        <div class="mt-20px grid gap-12px md:grid-cols-2">
          <a
            v-for="item in downloads"
            :key="item.title"
            :href="item.href"
            target="_blank"
            rel="noreferrer"
            class="group rounded-[28px] border border-white/10 bg-white/6 p-18px text-white no-underline transition-all duration-300 hover:border-[#f3c98b]/40 hover:bg-white/10"
          >
            <p class="text-12px uppercase tracking-3px text-[#ffd49f]">
              {{ item.title }}
            </p>
            <p class="mt-10px text-14px leading-24px text-[#dacfe6]">
              {{ item.description }}
            </p>
            <span class="mt-18px inline-flex items-center gap-8px text-15px font-700 text-[#fff1d2]">
              {{ item.label }}
              <span class="transition-transform duration-300 group-hover:translate-x-2px">
                →
              </span>
            </span>
          </a>
        </div>
      </section>

      <footer class="px-6px pb-8px pt-22px text-center text-13px leading-22px text-text-muted">
        Built for desktop coding sessions, release tracking, and a little more presence than a silent terminal tab.
      </footer>
    </main>
  </div>
</template>

<style scoped>
.landing-shell {
  position: relative;
  min-height: 100vh;
  overflow: hidden;
  background:
    radial-gradient(circle at top, rgba(244, 207, 146, 0.18), transparent 26%),
    radial-gradient(circle at bottom left, rgba(169, 86, 47, 0.16), transparent 30%),
    linear-gradient(180deg, #09070c 0%, #0f0b14 36%, #130f19 100%);
}

.landing-shell__grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(255, 255, 255, 0.05) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 255, 255, 0.05) 1px, transparent 1px);
  background-position: center;
  background-size: 36px 36px;
  mask-image: linear-gradient(180deg, rgba(0, 0, 0, 0.65), transparent 82%);
  opacity: 0.18;
  pointer-events: none;
}

.landing-shell__glow {
  position: absolute;
  width: 440px;
  height: 440px;
  border-radius: 999px;
  filter: blur(80px);
  opacity: 0.2;
  pointer-events: none;
}

.landing-shell__glow--left {
  top: 70px;
  left: -120px;
  background: rgba(243, 201, 139, 0.7);
}

.landing-shell__glow--right {
  right: -140px;
  bottom: 120px;
  background: rgba(191, 112, 62, 0.72);
}

.hero-visual__frame {
  overflow: hidden;
  aspect-ratio: 0.96;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.hero-visual__panel {
  position: relative;
  margin: -58px 12px 0;
  padding: 18px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 24px;
  background: linear-gradient(180deg, rgba(15, 11, 20, 0.85), rgba(10, 8, 15, 0.94));
  backdrop-filter: blur(18px);
}

.hero-chip {
  display: inline-flex;
  align-items: center;
  min-height: 34px;
  padding: 0 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.05);
  color: #fff4dd;
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

@media (max-width: 767px) {
  .hero-visual__panel {
    margin: -44px 8px 0;
    padding: 16px;
  }

  .landing-shell__glow {
    width: 280px;
    height: 280px;
    filter: blur(60px);
  }
}
</style>
