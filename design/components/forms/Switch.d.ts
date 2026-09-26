import type { CSSProperties } from 'react';
/** 40×22 on/off switch for boolean settings. */
export interface SwitchProps {
  checked?: boolean;
  onChange?: (next: boolean) => void;
  disabled?: boolean;
  id?: string;
  style?: CSSProperties;
}
export declare function Switch(props: SwitchProps): JSX.Element;
