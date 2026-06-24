import type { ButtonHTMLAttributes } from "react";

/** Toolbar/sidebar button with CSS tooltip (data-tooltip). */
export function TipButton({
  tip,
  tipBelow = false,
  className,
  children,
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & { tip: string; tipBelow?: boolean }) {
  return (
    <button
      {...rest}
      className={`has-tooltip${tipBelow ? " tooltip-below" : ""}${className ? ` ${className}` : ""}`}
      data-tooltip={tip}
    >
      {children}
    </button>
  );
}
