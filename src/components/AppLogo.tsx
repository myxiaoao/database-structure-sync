interface AppLogoProps {
  className?: string;
  size?: number;
}

export function AppLogo({ className = "", size = 32 }: AppLogoProps) {
  return (
    <img
      src="/app-icon.png"
      width={size}
      height={size}
      alt="Database Structure Sync"
      className={className}
      style={{ borderRadius: size * 0.18 }}
    />
  );
}
