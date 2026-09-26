import { Text } from 'ink';
import Spinner from 'ink-spinner'; // pass: Ink writes only the last frame when output is not interactive
export const A = () => <Text><Spinner type="dots" /> working</Text>;
