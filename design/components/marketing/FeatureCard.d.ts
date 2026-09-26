import type { ReactNode, CSSProperties } from 'react';
/** Feature grid tile. */
export interface FeatureCardProps {
  /** an Icon name (assets/icons/) */
  icon?: 'gpu' | 'eye' | 'api' | 'tray' | 'key' | 'box' | 'moon' | 'shield';
  title: ReactNode;
  /** icon tint — pick tones to make a grid colourful */
  tone?: 'running' | 'loading' | 'paused' | 'off' | 'error' | 'game';
  children?: ReactNode;
  style?: CSSProperties;
}
export declare function FeatureCard(props: FeatureCardProps): JSX.Element;
