/**
 * useAnimations.ts
 * Centralized Framer Motion animation patterns for consistent animations across the site
 * Eliminates duplication of animation logic and ensures visual consistency
 */

/**
 * Section entrance animation: fades in and slides up on view
 * Used for major page sections
 */
export const sectionAnimation = {
  initial: { opacity: 0, y: 30 },
  whileInView: { opacity: 1, y: 0 },
  viewport: { once: true, margin: "-50px" },
  transition: { delay: 0.2, duration: 0.6 },
};

/**
 * Header animation: fades in and slides down on page load
 * Used for section headers
 */
export const headerAnimation = {
  initial: { opacity: 0, y: -20 },
  whileInView: { opacity: 1, y: 0 },
  viewport: { once: true, margin: "-100px" },
};

/**
 * Card entrance animation: fades in with slight scale
 * Used for individual cards and components
 */
export const cardAnimation = (delay = 0) => ({
  initial: { opacity: 0, y: 20 },
  whileInView: { opacity: 1, y: 0 },
  viewport: { once: true },
  transition: { delay, duration: 0.5 },
});

/**
 * Card hover animation: lifts up on hover
 * Used for interactive cards
 */
export const cardHoverAnimation = {
  whileHover: { y: -5 },
  transition: { duration: 0.3 },
};

/**
 * Button hover animation: scales up with glow effect
 * Used for CTA buttons
 */
export const buttonHoverAnimation = (glowColor = "var(--color-rose-pine-iris)") => ({
  whileHover: {
    scale: 1.05,
    boxShadow: `0 0 30px -5px ${glowColor}`,
  },
  whileTap: { scale: 0.95 },
  transition: { duration: 0.2 },
});

/**
 * Button tap animation: scales down on click
 * Used for interactive buttons
 */
export const buttonTapAnimation = {
  whileTap: { scale: 0.95 },
};

/**
 * Shimmer effect animation: horizontal light sweep
 * Used for button overlays and special effects
 */
export const shimmerAnimation = {
  animate: { x: ["-100%", "100%"] },
  transition: {
    duration: 1.5,
    repeat: Infinity,
    repeatType: "loop" as const,
  },
};

/**
 * Fade in animation: simple opacity change
 * Used for text and content
 */
export const fadeAnimation = (delay = 0) => ({
  initial: { opacity: 0 },
  whileInView: { opacity: 1 },
  viewport: { once: true },
  transition: { delay, duration: 0.5 },
});

/**
 * Slide in from left animation
 * Used for asymmetric layouts
 */
export const slideInLeftAnimation = (delay = 0) => ({
  initial: { opacity: 0, x: -20 },
  whileInView: { opacity: 1, x: 0 },
  viewport: { once: true },
  transition: { delay, duration: 0.5 },
});

/**
 * Slide in from right animation
 * Used for asymmetric layouts
 */
export const slideInRightAnimation = (delay = 0) => ({
  initial: { opacity: 0, x: 20 },
  whileInView: { opacity: 1, x: 0 },
  viewport: { once: true },
  transition: { delay, duration: 0.5 },
});

/**
 * Scale animation: grows from 0 to 1
 * Used for metric displays and counters
 */
export const scaleAnimation = (delay = 0) => ({
  initial: { opacity: 0, scale: 0.95 },
  whileInView: { opacity: 1, scale: 1 },
  viewport: { once: true },
  transition: { delay, duration: 0.6, ease: "circOut" },
});

/**
 * Stagger container: animates children with delays
 * Used for lists and grids of items
 */
export const staggerContainer = {
  initial: { opacity: 0 },
  whileInView: { opacity: 1 },
  viewport: { once: true },
  transition: {
    staggerChildren: 0.1,
    delayChildren: 0.2,
  },
};

/**
 * Stagger item: used inside staggerContainer
 * Used for individual items in staggered lists
 */
export const staggerItem = {
  initial: { opacity: 0, y: 20 },
  whileInView: { opacity: 1, y: 0 },
  viewport: { once: true },
};

/**
 * Height animation: animates height for expanding content
 * Used for disclosure panels and expandable sections
 */
export const heightAnimation = {
  initial: { height: 0, opacity: 0 },
  animate: { height: "auto", opacity: 1 },
  exit: { height: 0, opacity: 0 },
  transition: { duration: 0.3 },
};

/**
 * Rotating animation: continuous rotation
 * Used for loaders and spinners
 */
export const rotatingAnimation = {
  animate: { rotate: 360 },
  transition: {
    duration: 2,
    repeat: Infinity,
    ease: "linear",
  },
};

/**
 * Pulse animation: grows and shrinks continuously
 * Used for attention-grabbing elements
 */
export const pulseAnimation = {
  animate: { scale: [1, 1.05, 1] },
  transition: {
    duration: 2,
    repeat: Infinity,
    ease: "easeInOut",
  },
};

/**
 * Float animation: gentle up and down movement
 * Used for floating elements and backgrounds
 */
export const floatAnimation = (duration = 6) => ({
  animate: { y: [0, -10, 0] },
  transition: {
    duration,
    repeat: Infinity,
    ease: "easeInOut",
  },
});

/**
 * Page transition: fade in and slight scale
 * Used for page changes
 */
export const pageTransitionAnimation = {
  initial: { opacity: 0, scale: 0.95 },
  animate: { opacity: 1, scale: 1 },
  transition: { duration: 0.3, ease: "easeOut" },
};

/**
 * Modal animation: scale up from center with backdrop
 * Used for modals and dialogs
 */
export const modalAnimation = {
  initial: { opacity: 0, scale: 0.9 },
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0.9 },
  transition: { duration: 0.3 },
};

/**
 * Tooltip animation: fades in and slides up
 * Used for tooltips and popovers
 */
export const tooltipAnimation = {
  initial: { opacity: 0, y: 5 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 5 },
  transition: { duration: 0.2 },
};
