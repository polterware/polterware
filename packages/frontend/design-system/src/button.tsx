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
  "pw-button inline-flex border border-transparent bg-transparent shrink-0 cursor-pointer items-center justify-center font-medium select-none outline-hidden transition-[background,border-color,color] duration-[var(--duration-control)] ease-[var(--ease-standard)] disabled:pointer-events-none disabled:cursor-not-allowed disabled:text-[var(--color-text-disabled)] [&_svg]:pointer-events-none [&_svg]:shrink-0";

/** Solid / textured variants use main-era material; ghost + icon stay flat with a full-round hover. */
const variantClass = {
  primary: "texture-button-material texture-button-material-primary",
  secondary: "texture-button-material texture-button-material-secondary",
  danger: "texture-button-material texture-button-material-destructive",
  brand: "texture-button-material texture-button-material-brand",
  ghost:
    "pw-button-flat text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]",
  icon:
    "pw-button-flat text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]",
} satisfies Record<ButtonVariant, string>;

const sizeClass = {
  sm: "h-[var(--control-height-sm)] gap-1.5 px-2.5 text-xs",
  md: "h-[var(--control-height-md)] gap-1.5 px-3 text-[13px]",
  icon: "h-[var(--control-height-sm)] w-[var(--control-height-sm)] p-0 [&_svg]:h-3.5 [&_svg]:w-3.5",
} satisfies Record<ButtonSize, string>;

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
    // Flat buttons hover as a full pill or circle; textured buttons keep the control radius.
    const radius = variant === "ghost" || variant === "icon" || size === "icon" ? "rounded-full" : "rounded-[var(--radius-control)]";
    return cn(base, variantClass[variant], sizeClass[size], radius, local.class);
  };
  return <button type="button" {...rest} class={className()} classList={local.classList} />;
}
