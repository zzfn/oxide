#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');
const os = require('os');

// 平台映射
const PLATFORMS = {
  darwin: {
    x64: 'oxide-x86_64-apple-darwin',
    arm64: 'oxide-aarch64-apple-darwin'
  },
  linux: {
    x64: 'oxide-x86_64-unknown-linux-gnu',
    arm64: 'oxide-aarch64-unknown-linux-gnu'
  },
  win32: {
    x64: 'oxide-x86_64-pc-windows-msvc.exe',
    arm64: 'oxide-aarch64-pc-windows-msvc.exe'
  }
};

function getPlatform() {
  const platform = os.platform();
  const arch = os.arch();

  const archMap = {
    x64: 'x64',
    arm64: 'arm64',
    arm: 'arm64'
  };

  const normalizedArch = archMap[arch] || arch;

  if (PLATFORMS[platform] && PLATFORMS[platform][normalizedArch]) {
    return PLATFORMS[platform][normalizedArch];
  }

  throw new Error(`Unsupported platform: ${platform}-${arch}`);
}

function getDownloadUrl(version, platform) {
  const baseUrl = 'https://github.com/zzfn/oxide/releases/download';
  const isWindows = platform.endsWith('.exe');

  if (isWindows) {
    return `${baseUrl}/v${version}/${platform}`;
  } else {
    return `${baseUrl}/v${version}/${platform}`;
  }
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);

    https.get(url, {
      headers: {
        'User-Agent': 'node-oxide-installer'
      }
    }, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        downloadFile(response.headers.location, dest)
          .then(resolve)
          .catch(reject);
        return;
      }

      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }

      response.pipe(file);

      file.on('finish', () => {
        file.close(() => resolve());
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
}

async function main() {
  try {
    const packageJson = require('../package.json');
    const version = packageJson.version;
    const platform = getPlatform();

    console.log(`Installing Oxide CLI v${version} for ${platform}...`);

    const binDir = path.join(__dirname, '..', 'bin');
    const binPath = path.join(binDir, 'oxide');

    // 确保 bin 目录存在
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    const url = getDownloadUrl(version, platform);
    const tempPath = path.join(binDir, platform);

    console.log(`Downloading from ${url}...`);
    await downloadFile(url, tempPath);

    // 移动到最终位置
    if (fs.existsSync(binPath)) {
      fs.unlinkSync(binPath);
    }

    fs.renameSync(tempPath, binPath);

    // 设置执行权限（非 Windows）
    if (process.platform !== 'win32') {
      fs.chmodSync(binPath, '755');
    }

    console.log('✓ Oxide CLI installed successfully!');
  } catch (error) {
    console.error('Failed to download pre-built binary:', error.message);
    console.log('\nFalling back to source installation...');
    console.log('Note: This requires Rust to be installed.');
    console.log('Install Rust from: https://rustup.rs/\n');

    try {
      // 尝试从源码构建
      execSync('cargo build --release', {
        cwd: path.join(__dirname, '..'),
        stdio: 'inherit'
      });

      const sourceBin = path.join(__dirname, '..', 'target', 'release', 'oxide');
      const destBin = path.join(__dirname, '..', 'bin', 'oxide');
      const binDir = path.dirname(destBin);

      if (!fs.existsSync(binDir)) {
        fs.mkdirSync(binDir, { recursive: true });
      }

      fs.copyFileSync(sourceBin, destBin);

      if (process.platform !== 'win32') {
        fs.chmodSync(destBin, '755');
      }

      console.log('\n✓ Oxide CLI built from source successfully!');
    } catch (buildError) {
      console.error('\n✗ Failed to build from source:', buildError.message);
      console.error('\nPlease install Rust from https://rustup.rs/ and try again.');
      process.exit(1);
    }
  }
}

main();
