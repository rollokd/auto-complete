import { PRODUCT_NAME } from "@/lib/constants";

export function Glyph({ size = 24 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <rect
        width="64"
        height="64"
        rx="14"
        className="fill-zinc-900 dark:fill-zinc-100"
      />
      <path
        d="M19 22L29 32L19 42M35 42H46"
        className="stroke-white dark:stroke-zinc-900"
        strokeWidth="5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export default function Lockup({ size = 72 }: { size?: number }) {
  return (
    <div className="flex flex-col items-center gap-3">
      <Glyph size={size} />
      <span className="text-xl font-semibold tracking-tight leading-none select-none">
        {PRODUCT_NAME}
      </span>
    </div>
  );
}
