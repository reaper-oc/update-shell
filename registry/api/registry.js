const packages = require('./packages.json');

module.exports = function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Content-Type', 'application/json');
  const list = packages.map(p => ({
    name: p.name,
    version: p.version,
    description: p.description,
    author: p.author,
    type: p.type,
    language: p.language || null,
    depends: p.depends || [],
    source_url: p.source_url || null,
    build_type: p.build_type || null,
  }));
  res.status(200).json(list);
};
