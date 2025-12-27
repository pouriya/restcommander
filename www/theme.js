/**
 * Theme Management Module
 * Handles dark/light theme switching using CSS custom properties
 */

const THEME_STORAGE_KEY = 'restcommander-theme';
const DEFAULT_THEME = 'dark';

/**
 * Get the current theme from localStorage or return default
 * @returns {string} Current theme ('dark' or 'light')
 */
export function getTheme() {
    try {
        const stored = localStorage.getItem(THEME_STORAGE_KEY);
        return stored === 'light' ? 'light' : DEFAULT_THEME;
    } catch (e) {
        // Fallback to default if localStorage is unavailable (e.g., incognito mode)
        return DEFAULT_THEME;
    }
}

/**
 * Save theme preference to localStorage
 * @param {string} theme - Theme to save ('dark' or 'light')
 */
function saveTheme(theme) {
    try {
        localStorage.setItem(THEME_STORAGE_KEY, theme);
    } catch (e) {
        // Silently fail if localStorage is unavailable
    }
}

/**
 * Update the theme toggle button icon based on current theme
 */
function updateToggleButtonIcon() {
    const toggleButton = document.getElementById('theme-toggle');
    if (!toggleButton) return;

    const currentTheme = getTheme();
    const icon = toggleButton.querySelector('svg');
    if (!icon) return;

    // Show sun icon when dark (to switch to light)
    // Show moon icon when light (to switch to dark)
    if (currentTheme === 'dark') {
        // Currently dark - show sun icon (clicking will switch to light)
        icon.innerHTML = '<path d="M8 11a3 3 0 1 1 0-6 3 3 0 0 1 0 6zm0 1a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM8 0a.5.5 0 0 1 .5.5v2a.5.5 0 0 1-1 0v-2A.5.5 0 0 1 8 0zm0 13a.5.5 0 0 1 .5.5v2a.5.5 0 0 1-1 0v-2A.5.5 0 0 1 8 13zm8-5a.5.5 0 0 1-.5.5h-2a.5.5 0 0 1 0-1h2a.5.5 0 0 1 .5.5zM3 8a.5.5 0 0 1-.5.5h-2a.5.5 0 0 1 0-1h2A.5.5 0 0 1 3 8zm10.657-5.657a.5.5 0 0 1 0 .707l-1.414 1.415a.5.5 0 1 1-.707-.708l1.414-1.414a.5.5 0 0 1 .707 0zm-9.193 9.193a.5.5 0 0 1 0 .707L3.05 13.657a.5.5 0 0 1-.707-.707l1.414-1.414a.5.5 0 0 1 .707 0zm9.193 2.121a.5.5 0 0 1-.707 0l-1.414-1.414a.5.5 0 0 1 .707-.707l1.414 1.414a.5.5 0 0 1 0 .707zM4.464 4.465a.5.5 0 0 1-.707 0L2.343 3.05a.5.5 0 1 1 .707-.707l1.414 1.414a.5.5 0 0 1 0 .708z"/>';
    } else {
        // Currently light - show moon icon (clicking will switch to dark)
        icon.innerHTML = '<path d="M6 .278a.768.768 0 0 1 .08.858 7.208 7.208 0 0 0-.878 3.46c0 4.021 3.278 7.277 7.318 7.277.527 0 1.04-.055 1.533-.16a.787.787 0 0 1 .81.316.733.733 0 0 1-.031.893A8.349 8.349 0 0 1 8.344 16C3.734 16 0 12.286 0 7.71 0 4.266 2.114 1.312 5.124.06A.752.752 0 0 1 6 .278z"/>';
    }
}

/**
 * Initialize theme on page load
 * Loads theme from localStorage, sets data-theme attribute, and updates button icon
 */
export function initTheme() {
    const theme = getTheme();
    document.documentElement.setAttribute('data-theme', theme);
    updateToggleButtonIcon();
}

/**
 * Toggle between dark and light themes
 * Saves preference to localStorage and updates UI
 */
export function toggleTheme() {
    const currentTheme = getTheme();
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    
    document.documentElement.setAttribute('data-theme', newTheme);
    saveTheme(newTheme);
    updateToggleButtonIcon();
}

