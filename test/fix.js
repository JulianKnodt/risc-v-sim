const fs = require('fs');

fs.readdirSync('.')
  .filter(it => it.endsWith('.asm'))
  .forEach(file => {
    console.log(file);
    let contents = fs.readFileSync(file)
      .toString()
      .split('\n')
      .map(it => it.trim())
      .join('\n')
      .replace(/\$/g, '');

    fs.writeFileSync(file, contents);
  });
