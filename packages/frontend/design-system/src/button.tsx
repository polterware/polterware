import type { JSX } from "solid-js";
import { splitProps } from "solid-js";
import "./button.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "brand" | "icon";
export type ButtonSize = "sm" | "md" | "icon";

export type ButtonProps = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
};

const base =
  "inline-flex shrink-0 cursor-pointer items-center justify-center font-medium select-none outline-hidden transition-[background,border-color,color] duration-[var(--duration-control)] ease-[var(--ease-standard)] disabled:pointer-events-none disabled:cursor-not-allowed disabled:text-[var(--color-text-disabled)] [&_svg]:pointer-events-none [&_svg]:shrink-0";

/** Solid / textured variants use main-era material; ghost + icon stay flat. */
const variantClass: Record<ButtonVariant, string> = {
  primary: "texture-button-material texture-button-material-primary",
  secondary: "texture-button-material texture-button-material-secondary",
  danger: "texture-button-material texture-button-material-destructive",
  brand: "texture-button-material texture-button-material-brand",
  ghost:
    "bg-transparent text-[var(--color-text-secondary)] border border-transparent hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]",
  icon:
    "bg-transparent text-[var(--color-text-secondary)] border border-transparent hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]",
};

const sizeClass: Record<ButtonSize, string> = {
  sm: "h-[var(--control-height-sm)] gap-1.5 rounded-[var(--radius-control)] px-2.5 text-xs",
  md: "h-[var(--control-height-md)] gap-1.5 rounded-[var(--radius-control)] px-3 text-[13px]",
  icon: "h-[var(--control-height-sm)] w-[var(--control-height-sm)] rounded-[var(--radius-control)] p-0 [&_svg]:h-3.5 [&_svg]:w-3.5",
};

function cn(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

/**
 * Product button — texture material for solid variants; ghost/icon flat.
 * Default is secondary (neutral). Primary is pastel cyan and must be opt-in
 * for PageHeader top-right CTAs only.
 */
export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, ["variant", "size", "class", "classList"]);
  const className = () => {
    const variant = local.variant ?? "secondary";
    const size = local.size ?? (variant === "icon" ? "icon" : "md");
    return cn(base, variantClass[variant], sizeClass[size], local.class);
  };
  return <button type="button" {...rest} class={className()} classList={local.classList} />;
}
