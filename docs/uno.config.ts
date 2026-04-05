import {
  defineConfig,
  presetAttributify,
  presetIcons,
  presetTypography,
  presetWebFonts,
  presetWind3,
  transformerDirectives,
  transformerVariantGroup,
} from 'unocss'

export default defineConfig({
  safelist: [
    // ListProjects 动态图标 (添加新项目图标时需同步更新)
    'i-carbon-machine-learning-model',
    'i-carbon-face-wink',
    'i-carbon-blog',
    'i-carbon-image',
  ],
  shortcuts: {
    // TNO 风格按钮
    'tno-btn': 'bg-tno-bg-90 border-2px border-tno-border text-tno-text-muted cursor-pointer transition-all duration-200',
    'tno-btn-hover': 'text-tno-accent border-2px bg-tno-hover-bg border-tno-border-hover',
    'tno-btn-active': 'bg-tno-active-bg border-2px border-tno-active-border text-tno-active-text',
  },
  theme: {
    colors: {
      // 原有颜色
      desc: '#61666D',
      title: '#18191C',
      primary: '#FF6699',
      gray: '#F6F7F8',
      bdline: '#E3E5E7',

      // 哥特暗黑风配色
      gothic: {
        bg: '#0d0a0e',
        surface: '#1a1520',
        elevated: '#251d2a',
        border: '#3d2d4a',
      },
      blood: {
        DEFAULT: '#8b0000',
        light: '#b22222',
        glow: '#ff4444',
      },
      violet: {
        DEFAULT: '#4a0080',
        light: '#7b2cbf',
        glow: '#9d4edd',
      },
      text: {
        primary: '#e8e0f0',
        secondary: '#a89bb8',
        muted: '#6b5a7a',
      },
      dialogue: {
        bg: 'rgba(13, 10, 14, 0.95)',
        border: '#7b2cbf',
        nameBg: '#4a0080',
      },
      // TNO 风格配色
      tno: {
        // 基础色值 (纯色)
        primary: '#00d4aa',
        accent: '#00ffcc',
        dim: '#006b55',

        // 背景色 (带透明度)
        bg: {
          DEFAULT: 'rgba(10, 18, 26, 0.95)',
          90: 'rgba(10, 18, 26, 0.9)',
          98: 'rgba(10, 18, 26, 0.98)',
          solid: '#0a121a',
        },

        // hover 背景
        hover: {
          bg: 'rgba(0, 212, 170, 0.15)',
        },

        // active 状态 (红色主题)
        active: {
          bg: 'rgba(139, 0, 0, 0.15)',
          border: '#ff4444',
          text: '#ff6666',
          glow: 'rgba(255, 68, 68, 0.5)',
          glowSm: 'rgba(255, 68, 68, 0.3)',
          textGlow: 'rgba(255, 68, 68, 0.6)',
        },

        // 边框色
        border: {
          DEFAULT: '#00d4aa',
          hover: '#00ffcc',
          dim: '#006b55',
        },

        // 文字色
        text: {
          DEFAULT: '#e8f0f0',
          muted: '#8ba8a8',
          accent: '#00ffcc',
        },

        // 发光效果 (box-shadow 用)
        glow: {
          sm: 'rgba(0, 212, 170, 0.3)',
          DEFAULT: 'rgba(0, 212, 170, 0.4)',
          md: 'rgba(0, 212, 170, 0.5)',
          lg: 'rgba(0, 212, 170, 0.7)',
        },

        // 文字阴影 (text-shadow 用)
        textGlow: {
          sm: 'rgba(0, 255, 204, 0.4)',
          DEFAULT: 'rgba(0, 255, 204, 0.5)',
          lg: 'rgba(0, 255, 204, 0.6)',
        },

        // 扫描线纹理
        scanline: {
          light: 'rgba(0, 212, 170, 0.02)',
          DEFAULT: 'rgba(0, 212, 170, 0.03)',
        },
      },
    },
  },
  presets: [
    presetWind3(),
    presetAttributify(),
    presetIcons(),
    presetTypography(),
    presetWebFonts(),
  ],
  transformers: [
    transformerDirectives(),
    transformerVariantGroup(),
  ],
})
