import type { CSSProperties } from 'react';
/** Line icon (Lucide) tinted by CSS. Intentional addition — the site ships inline stroke SVGs of the same style. */
export interface IconProps {
  /** lucide icon name, e.g. "cpu", "gamepad-2", "download" */
  name: string;
  size?: number;
  /** any CSS colour; defaults to currentColor */
  color?: string;
  style?: CSSProperties;
}
export declare function Icon(props: IconProps): JSX.Element;
