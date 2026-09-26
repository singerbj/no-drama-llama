import type { ReactNode, CSSProperties, MouseEventHandler } from 'react';
/**
 * Compact settings-window button (6px radius, 5px 14px padding).
 * @startingPoint section="Core" subtitle="Default, primary, danger and tab buttons" viewport="700x260"
 */
export interface ButtonProps {
  /** default = neutral panel button; primary = mint fill; danger = tomato outline; tab = side-nav item */
  variant?: 'default' | 'primary' | 'danger' | 'tab';
  /** tab variant only: selected state (mint-soft fill, full 1px border, mint text) */
  active?: boolean;
  disabled?: boolean;
  type?: 'button' | 'submit';
  onClick?: MouseEventHandler<HTMLButtonElement>;
  style?: CSSProperties;
  children?: ReactNode;
}
export declare function Button(props: ButtonProps): JSX.Element;
