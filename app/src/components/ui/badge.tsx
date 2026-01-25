import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--accent)] focus:ring-offset-2",
  {
    variants: {
      variant: {
        default:
          "border-transparent bg-[var(--accent)] text-white",
        secondary:
          "border-transparent bg-[var(--bg-tertiary)] text-[var(--text-primary)]",
        destructive:
          "border-transparent bg-red-500/10 text-red-500 border-red-500/20",
        outline: "text-[var(--text-secondary)] border-[var(--border-light)]",
        success:
          "border-transparent bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
        warning:
          "border-transparent bg-amber-500/10 text-amber-500 border-amber-500/20",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
