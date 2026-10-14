import type { JSX } from "solid-js";
import { splitProps } from "solid-js";
import "./button.css";
import "./control-button.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "brand" | "icon";
export type ButtonSize = "sm" | "md" | "icon";

export type ButtonProps = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
};

const base =
  "inline-flex shrink-0 cursor-pointer items-center justify-center font-medium select-none [&_svg]:pointer-events-none [&_svg]:shrink-0";

const flat = "pw-button pw-button-flat border border-transparent bg-transparent outline-hidden transition-[background,border-color,color] duration-[var(--duration-control)] ease-[var(--ease-standard)] disabled:pointer-events-none disabled:cursor-not-allowed disabled:text-[var(--color-text-disabled)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]";
/** Solid variants share the opaque ControlButton chrome; ghost + icon stay flat with a full-round hover. */
const control = "pw-control-button control-field control-field-md gap-1.5";

const variantClass = {
  primary: control,
  secondary: control,
  danger: control,
  brand: control,
  ghost: flat,
  icon: flat,
} satisfies Record<ButtonVariant, string>;

const tone = {
  primary: "brand",
  secondary: "secondary",
  danger: "danger",
  brand: "brand",
} as const;

const sizeClass = {
  sm: "h-[var(--control-height-sm)] gap-1.5 px-2.5 text-xs",
  md: "h-[var(--control-height-md)] gap-1.5 px-3 text-[13px]",
  icon: "h-[var(--control-height-sm)] w-[var(--control-height-sm)] p-0 [&_svg]:h-3.5 [&_svg]:w-3.5",
} satisfies Record<ButtonSize, string>;

function toneOf(variant: ButtonVariant = "secondary") {
  return variant === "ghost" || variant === "icon" ? undefined : tone[variant];
}

function cn(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

/**
 * Product button — ControlButton chrome for solid variants; ghost/icon flat.
 * Default is secondary (neutral). Primary is pastel cyan and must be opt-in
 * for PageHeader top-right CTAs only.
 */
export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, ["variant", "size", "class", "classList"]);
  const className = () => buttonClassName(local.variant, local.size, local.class);
  return <button type="button" {...rest} data-tone={toneOf(local.variant)} class={className()} classList={local.classList} />;
}

/** Native link with the same materials and sizing as Button. */
export type LinkButtonProps = JSX.AnchorHTMLAttributes<HTMLAnchorElement> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
};

function buttonClassName(variant: ButtonVariant = "secondary", size?: ButtonSize, className?: string) {
  const actualSize = size ?? (variant === "icon" ? "icon" : "md");
  const radius = variant === "ghost" || variant === "icon" || actualSize === "icon"
    ? "rounded-full" : "rounded-[var(--radius-control)]";
  return cn(base, variantClass[variant], sizeClass[actualSize], radius, className);
}

export function LinkButton(props: LinkButtonProps) {
  const [local, rest] = splitProps(props, ["variant", "size", "class", "classList"]);
  return <a {...rest} data-tone={toneOf(local.variant)} class={buttonClassName(local.variant, local.size, local.class)} classList={local.classList} />;
}
