<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import main from '@/assets/images/main.png'

const repositoryHref = 'https://github.com/Panzer-Jack/Copiwaifu'
const latestReleaseHref = `${repositoryHref}/releases/latest`
const easyLive2dHref = 'https://github.com/Panzer-Jack/easy-live2d'
const Sprite = import.meta.env.SSR
  ? null
  : defineAsyncComponent(() => import('@/components/sprite.vue'))
</script>

<template>
  <div class="hero-page">
    <div
      class="hero-page__bg"
      :style="{ backgroundImage: `url(${main})` }"
    />
    <div class="hero-page__veil" />
    <div class="hero-page__mist hero-page__mist--top" />
    <div class="hero-page__mist hero-page__mist--bottom" />

    <main class="hero-layout">
      <header class="hero-layout__header">
        <div>
          <p class="hero-layout__brand font-display">
            Copiwaifu
          </p>
          <p class="hero-layout__brand-sub">
            Live2D AI desktop companion
          </p>
        </div>
      </header>

      <div class="hero-layout__content">
        <section class="hero-card">
          <p class="hero-card__eyebrow font-display">
            Desktop presence for AI sessions
          </p>

          <h1 class="hero-card__project font-display">
            Copiwaifu
          </h1>

          <p class="hero-card__subtitle font-display">
            Keep your AI workflow visible, softly and clearly.
          </p>

          <p class="hero-card__lead">
            Copiwaifu is built on top of
            <a
              :href="easyLive2dHref"
              target="_blank"
              rel="noreferrer"
              class="hero-card__inline-link"
            >
              easy-live2d
            </a>,
            turning thinking, tool use, completion, and attention states into a calm Live2D presence that stays on the desktop without becoming visual noise.
          </p>

          <div class="hero-card__actions">
            <a
              :href="latestReleaseHref"
              target="_blank"
              rel="noreferrer"
              class="hero-card__button hero-card__button--primary"
            >
              Download
            </a>
            <a
              :href="repositoryHref"
              target="_blank"
              rel="noreferrer"
              class="hero-card__button hero-card__button--secondary"
            >
              GitHub
            </a>
          </div>
        </section>

        <section class="hero-stage" aria-label="Copiwaifu Live2D preview">
            <component :is="Sprite" v-if="Sprite" />
        </section>
      </div>
    </main>
  </div>
</template>

<style scoped>
.hero-page {
  position: relative;
  min-height: 100vh;
  min-height: 100svh;
  overflow: hidden;
  background: #0d1420;
}

.hero-page__bg,
.hero-page__veil,
.hero-page__mist {
  position: fixed;
  inset: 0;
  pointer-events: none;
}

.hero-page__bg {
  background-position: center center;
  background-repeat: no-repeat;
  background-size: cover;
  transform: scale(1.04);
}

.hero-page__veil {
  background:
    linear-gradient(90deg, rgba(10, 18, 30, 0.78) 0%, rgba(10, 18, 30, 0.54) 34%, rgba(10, 18, 30, 0.18) 60%, rgba(10, 18, 30, 0.32) 100%),
    linear-gradient(180deg, rgba(8, 14, 24, 0.28) 0%, rgba(8, 14, 24, 0.08) 26%, rgba(8, 14, 24, 0.36) 100%);
}

.hero-page__mist {
  border-radius: 999px;
  filter: blur(80px);
  opacity: 0.5;
}

.hero-page__mist--top {
  top: -12%;
  left: -8%;
  width: 34vw;
  height: 34vw;
  min-width: 260px;
  min-height: 260px;
  background: rgba(171, 214, 255, 0.28);
}

.hero-page__mist--bottom {
  top: auto;
  bottom: -18%;
  left: 16%;
  width: 44vw;
  height: 24vw;
  min-width: 320px;
  min-height: 220px;
  background: rgba(255, 255, 255, 0.18);
}

.hero-layout {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  min-height: 100svh;
  max-width: 1280px;
  margin: 0 auto;
  padding: 24px;
}

.hero-layout__content {
  display: flex;
  flex: 1;
  align-items: stretch;
  justify-content: space-between;
  gap: clamp(32px, 5vw, 88px);
}

.hero-layout__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.hero-layout__brand {
  color: rgba(236, 246, 255, 0.96);
  font-size: 30px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-shadow: 0 8px 30px rgba(8, 16, 27, 0.35);
}

.hero-layout__brand-sub {
  margin-top: 4px;
  color: rgba(223, 237, 249, 0.7);
  font-size: 13px;
  line-height: 1.6;
}

.hero-card {
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: min(100%, 560px);
  flex: 0 1 560px;
  padding: 24px 0 36px;
}

.hero-stage {
  display: flex;
  flex: 1;
  align-items: flex-end;
  justify-content: flex-end;
  padding: 48px 0 36px;
}

.hero-stage__shell {
  position: relative;
  width: min(100%, 440px);
  height: min(76vh, 680px);
  min-height: 440px;
  border: 1px solid rgba(216, 233, 247, 0.14);
  border-radius: 36px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.12) 0%, rgba(10, 20, 32, 0.08) 100%),
    rgba(9, 18, 29, 0.12);
  backdrop-filter: blur(14px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.12),
    0 24px 60px rgba(4, 11, 19, 0.18);
}

.hero-card__eyebrow {
  color: rgba(188, 220, 247, 0.9);
  font-size: 12px;
  letter-spacing: 0.24em;
  text-transform: uppercase;
}

.hero-card__project {
  margin-top: 18px;
  color: #f3f9ff;
  font-size: clamp(72px, 11vw, 156px);
  line-height: 0.86;
  font-weight: 700;
  letter-spacing: 0.01em;
  text-shadow:
    0 12px 36px rgba(8, 16, 28, 0.38),
    0 2px 0 rgba(255, 255, 255, 0.12);
}

.hero-card__subtitle {
  max-width: 480px;
  margin-top: 30px;
  color: rgba(230, 242, 252, 0.92);
  font-size: clamp(20px, 2.6vw, 34px);
  line-height: 1.15;
  font-weight: 500;
  text-wrap: balance;
  text-shadow: 0 8px 24px rgba(9, 16, 28, 0.28);
}

.hero-card__lead {
  max-width: 470px;
  margin-top: 22px;
  padding: 18px 20px;
  border: 1px solid rgba(209, 229, 247, 0.14);
  border-radius: 24px;
  background: rgba(12, 22, 35, 0.2);
  color: rgba(231, 242, 252, 0.78);
  font-size: 15px;
  line-height: 1.85;
  backdrop-filter: blur(14px);
  box-shadow: 0 18px 40px rgba(5, 12, 20, 0.16);
}

.hero-card__inline-link {
  color: rgba(243, 249, 255, 0.96);
  text-decoration: none;
  border-bottom: 1px solid rgba(214, 232, 248, 0.5);
  transition: border-color 0.24s ease, color 0.24s ease;
}

.hero-card__inline-link:hover {
  color: #ffffff;
  border-color: rgba(255, 255, 255, 0.92);
}

.hero-card__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 26px;
}

.hero-card__button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 48px;
  padding: 0 22px;
  border-radius: 999px;
  text-decoration: none;
  transition: transform 0.24s ease, background 0.24s ease, border-color 0.24s ease;
}

.hero-card__button:hover {
  transform: translateY(-1px);
}

.hero-card__button--primary {
  background: rgba(121, 183, 239, 0.92);
  color: #ffffff;
  font-weight: 700;
  box-shadow: 0 16px 34px rgba(77, 146, 207, 0.22);
}

.hero-card__button--secondary {
  border: 1px solid rgba(216, 232, 246, 0.18);
  background: rgba(255, 255, 255, 0.1);
  color: rgba(240, 247, 255, 0.92);
  font-weight: 600;
  backdrop-filter: blur(12px);
}

@media (max-width: 980px) {
  .hero-layout {
    padding: 18px;
  }

  .hero-layout__content {
    flex-direction: column;
    gap: 16px;
  }

  .hero-card {
    width: 100%;
    max-width: 100%;
    justify-content: flex-end;
    padding: 20px 0 28px;
  }

  .hero-stage {
    flex: none;
    align-items: center;
    justify-content: center;
    padding: 0 0 20px;
  }

  .hero-stage__shell {
    width: min(100%, 420px);
    height: min(62vh, 520px);
    min-height: 360px;
  }

  .hero-card__project {
    font-size: clamp(56px, 18vw, 108px);
  }

  .hero-card__subtitle {
    max-width: 100%;
    font-size: clamp(18px, 5vw, 28px);
  }

  .hero-card__lead {
    max-width: 100%;
  }
}

@media (prefers-reduced-motion: reduce) {
  .hero-card__button,
  .hero-card__inline-link {
    transition: none;
  }
}
</style>
