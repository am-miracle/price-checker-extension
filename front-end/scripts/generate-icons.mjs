import sharp from 'sharp';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const sizes = [16, 48, 128];
const svgPath = join(__dirname, '../public/icons/icon.svg');
const outputDir = join(__dirname, '../public/icons');

async function generateIcons() {
  console.log('Generating extension icons...\n');

  for (const size of sizes) {
    const outputPath = join(outputDir, `icon${size}.png`);

    try {
      await sharp(svgPath)
        .resize(size, size)
        .png()
        .toFile(outputPath);

      console.log(`✓ Generated ${size}x${size} icon: icon${size}.png`);
    } catch (error) {
      console.error(`✗ Failed to generate ${size}x${size} icon:`, error.message);
      process.exit(1);
    }
  }

  console.log('\n✓ All icons generated successfully!');
}

generateIcons();
