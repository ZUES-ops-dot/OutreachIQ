import type { Config } from 'tailwindcss'

const config: Config = {
  content: [
    './pages/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
    './app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        // Nord-based eye-friendly palette
        nord: {
          // Polar Night (backgrounds)
          bg: '#2e3440',
          surface: '#3b4252',
          elevated: '#434c5e',
          highlight: '#4c566a',
          // Snow Storm (text)
          text: '#eceff4',
          'text-secondary': '#e5e9f0',
          'text-muted': '#d8dee9',
          // Frost (accents)
          frost1: '#8fbcbb',  // Teal
          frost2: '#88c0d0',  // Cyan
          frost3: '#81a1c1',  // Blue (primary)
          frost4: '#5e81ac',  // Deep blue
          // Aurora (status colors)
          success: '#a3be8c',  // Sage green
          warning: '#ebcb8b',  // Sand yellow
          error: '#bf616a',    // Soft red
          purple: '#b48ead',   // Soft purple
        },
        stone: {
          50: '#fafaf9',
          100: '#f5f5f4',
          200: '#e7e5e4',
          300: '#d6d3d1',
          400: '#a8a29e',
          500: '#78716c',
          600: '#57534e',
          700: '#44403c',
          800: '#292524',
          900: '#1c1917',
        }
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
export default config
