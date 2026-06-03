const fs = require("fs");
const path = require("path");
const targetPath = path.join(
  __dirname,
  "frontend",
  "src",
  "generated_large.ts",
);
let content = `// Auto-generated large file for performance testing\n\n`;
for (let i = 0; i < 25; i++) content += `
export interface GeneratedInterface${i} {
    id: number;
    name: string;
    isActive: boolean;
    data: any[];
}

export const generatedFunction${i} = (item: GeneratedInterface${i}): string => {
    if (item.isActive) {
        return item.name.toUpperCase();
    }
    return "inactive";
};

`;
fs.writeFileSync(targetPath, content, "utf8");
console.log(`Generated large file at ${targetPath}`);
