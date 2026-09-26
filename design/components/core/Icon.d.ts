import type { CSSProperties } from 'react';
/** One of No Drama Llama's own line icons (assets/icons/), tinted by CSS. */
export interface IconProps {
  /** a file name in assets/icons/ */
  name: 'api' | 'box' | 'chevron-down' | 'eye' | 'gpu' | 'key' | 'moon' | 'shield' | 'theme' | 'tray' | 'windows';
  size?: number;
  /** any CSS colour; defaults to currentColor */
  color?: string;
  style?: CSSProperties;
}
export declare function Icon(props: IconProps): JSX.Element;
