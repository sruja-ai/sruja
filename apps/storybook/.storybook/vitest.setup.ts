import { setProjectAnnotations } from '@storybook/react-vite';
import * as projectAnnotations from './preview';
import * as a11yAnnotations from '@storybook/addon-a11y/preview';

// This is an important step to apply the right configuration when testing your stories.
// More info at: https://storybook.js.org/docs/api/portable-stories/portable-stories-vitest#setprojectannotations
setProjectAnnotations([projectAnnotations, a11yAnnotations]);
