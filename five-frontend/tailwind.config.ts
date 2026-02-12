import type { Config } from 'tailwindcss';

const config: Config = {
  content: [
    './src/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        'rose-pine-base': 'var(--rp-base)',
        'rose-pine-surface': 'var(--rp-surface)',
        'rose-pine-overlay': 'var(--rp-overlay)',
        'rose-pine-muted': 'var(--rp-muted)',
        'rose-pine-subtle': 'var(--rp-subtle)',
        'rose-pine-text': 'var(--rp-text)',
        'rose-pine-love': 'var(--rp-love)',
        'rose-pine-gold': 'var(--rp-gold)',
        'rose-pine-rose': 'var(--rp-rose)',
        'rose-pine-pine': 'var(--rp-pine)',
        'rose-pine-foam': 'var(--rp-foam)',
        'rose-pine-iris': 'var(--rp-iris)',
        'rose-pine-hl-low': 'var(--rp-hl-low)',
        'rose-pine-hl-med': 'var(--rp-hl-med)',
        'rose-pine-hl-high': 'var(--rp-hl-high)',
      },
      animation: {
        'float': 'float 6s ease-in-out infinite',
        'pulse-glow': 'pulse-glow 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'glow': 'glow 2s ease-in-out infinite alternate',
        'orb-1': 'orb-move-1 20s infinite ease-in-out',
        'orb-2': 'orb-move-2 25s infinite ease-in-out 2s',
        'orb-3': 'orb-move-3 22s infinite ease-in-out 1s',
        'orb-4': 'orb-move-4 28s infinite ease-in-out 3s',
        'orb-center': 'orb-center 30s infinite ease-in-out',
      },
      keyframes: {
        'float': {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        'pulse-glow': {
          '0%, 100%': {
            opacity: '1',
            boxShadow: '0 0 15px var(--rp-rose)',
          },
          '50%': {
            opacity: '0.7',
            boxShadow: '0 0 5px var(--rp-rose)',
          },
        },
        'glow': {
          'from': {
            boxShadow: '0 0 5px var(--rp-iris), 0 0 10px var(--rp-iris)',
          },
          'to': {
            boxShadow: '0 0 10px var(--rp-love), 0 0 20px var(--rp-love)',
          },
        },
        'orb-move-1': {
          '0%': { transform: 'translate(0, 0) scale(1)' },
          '33%': { transform: 'translate(100px, -50px) scale(1.2)' },
          '66%': { transform: 'translate(-50px, 100px) scale(0.9)' },
          '100%': { transform: 'translate(0, 0) scale(1)' },
        },
        'orb-move-2': {
          '0%': { transform: 'translate(0, 0) scale(1)' },
          '33%': { transform: 'translate(-100px, 50px) scale(0.8)' },
          '66%': { transform: 'translate(50px, -100px) scale(1.1)' },
          '100%': { transform: 'translate(0, 0) scale(1)' },
        },
        'orb-move-3': {
          '0%': { transform: 'translate(0, 0) scale(1)' },
          '33%': { transform: 'translate(80px, -60px) scale(1.3)' },
          '66%': { transform: 'translate(-40px, 40px) scale(0.9)' },
          '100%': { transform: 'translate(0, 0) scale(1)' },
        },
        'orb-move-4': {
          '0%': { transform: 'translate(0, 0) scale(1)' },
          '33%': { transform: 'translate(-90px, 70px) scale(0.9)' },
          '66%': { transform: 'translate(45px, -30px) scale(1.2)' },
          '100%': { transform: 'translate(0, 0) scale(1)' },
        },
        'orb-center': {
          '0%': { transform: 'translate(0, 0) scale(1)', opacity: '0.1' },
          '33%': { transform: 'translate(200px, -150px) scale(1.1)', opacity: '0.2' },
          '66%': { transform: 'translate(-200px, 150px) scale(1)', opacity: '0.1' },
          '100%': { transform: 'translate(0, 0) scale(1)', opacity: '0.1' },
        },
      },
    },
  },
  plugins: [],
};

export default config;
