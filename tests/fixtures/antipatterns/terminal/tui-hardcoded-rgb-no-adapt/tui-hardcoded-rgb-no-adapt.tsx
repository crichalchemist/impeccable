import { Text } from 'ink';
export const A = () => <Text color="#ff0080">mu</Text>; // flag
export const B = () => <Text color="cyan">mu</Text>; // pass
import chalk from 'chalk';
export const warn = chalk.bold.hex('#ff00aa')('warn'); // flag: a literal after a chained style
export const themed = (theme) => chalk.hex(theme.primary)('ok'); // pass: a computed argument
