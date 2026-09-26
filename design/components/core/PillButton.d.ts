import type { ReactNode, CSSProperties, MouseEventHandler } from 'react';
/** Marketing-site call-to-action pill. */
export interface PillButtonProps {
  variant?: 'primary' | 'ghost';
  size?: 'md' | 'sm';
  /** renders an <a> when set */
  href?: string;
  /** leading 18px icon, e.g. <Icon name="download" /> */
  icon?: ReactNode;
  onClick?: MouseEventHandler;
  style?: CSSProperties;
  children?: ReactNode;
}
export declare function PillButton(props: PillButtonProps): JSX.Element;
