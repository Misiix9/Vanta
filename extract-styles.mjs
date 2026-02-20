import fs from 'fs';
import path from 'path';

const SVELTE_FILES = [
    'src/routes/+page.svelte',
    'src/lib/components/ScriptResultItem.svelte',
    'src/lib/components/SearchInput.svelte',
    'src/lib/components/FileIcon.svelte',
    'src/lib/components/SettingsView.svelte',
    'src/lib/components/StatusBar.svelte',
    'src/lib/components/ResultItem.svelte',
    'src/lib/components/ResultsList.svelte'
];

let globalStyles = '\n/* --- EXTRACTED COMPONENT STYLES --- */\n';

for (const file of SVELTE_FILES) {
    let content = fs.readFileSync(file, 'utf8');
    const styleMatch = content.match(/<style>([\s\S]*?)<\/style>/);
    if (styleMatch) {
        globalStyles += `\n/* From ${file} */\n${styleMatch[1]}\n`;
        content = content.replace(/<style>[\s\S]*?<\/style>/, '');
        fs.writeFileSync(file, content);
    }
}

fs.appendFileSync('src/app.css', globalStyles);
console.log('Successfully extracted styles to app.css');
