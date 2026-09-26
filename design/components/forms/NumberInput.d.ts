import type { ChangeEventHandler, CSSProperties } from 'react';
/** Number field for ports, thresholds, seconds. */
export interface NumberInputProps {
  value?: number | string;
  onChange?: ChangeEventHandler<HTMLInputElement>;
  min?: number;
  max?: number;
  step?: number;
  width?: number | string;
  invalid?: boolean;
  style?: CSSProperties;
  [key: string]: unknown;
}
export declare function NumberInput(props: NumberInputProps): JSX.Element;
