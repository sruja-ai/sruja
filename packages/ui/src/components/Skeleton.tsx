export type SkeletonProps = {
  className?: string
}

export function Skeleton({ className }: SkeletonProps) {
  return <div className={["animate-pulse bg-[var(--color-neutral-100)]", className || ""].join(" ")} />
}

