const packages = require('../packages.json');

module.exports = function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Content-Type', 'application/json');

  const { name } = req.query;
  const pkg = packages.find(p => p.name === name);

  if (!pkg) {
    return res.status(404).json({ error: 'package not found' });
  }

  res.status(200).json(pkg);
};
