import { Text } from 'ink';
export const A = ({ t }) => <Text>{t.slice(0, 20) + '...'}</Text>; // flag
export const B = ({ t }) => <Text>{t}</Text>; // pass
