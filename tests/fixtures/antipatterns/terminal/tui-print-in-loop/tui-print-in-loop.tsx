import { render, Text } from 'ink';
console.log('debug'); // flag: this file turns patchConsole off
export const A = () => <Text>mu</Text>;
render(<A />, { patchConsole: false });
