<img src="https://ollama.com/assets/library/qwen3.5/b6074711-2930-4f09-93b7-4f65ae7a3576" width="360" />

Ces derniers mois, nous avons intensifié nos efforts sur le développement de modèles de fondation offrant une utilité et des performances exceptionnelles. Qwen3.5 représente une avancée significative, en intégrant des percées en apprentissage multimodal, en efficacité architecturale, en mise à l'échelle de l'apprentissage par renforcement et en accessibilité mondiale pour donner aux développeurs et aux entreprises une capacité et une efficacité sans précédent.

## Points forts

Qwen3.5 apporte les améliorations suivantes :

- **Fondation Vision-Langage Unifiée** : Un entraînement à fusion précoce sur des tokens multimodaux atteint la parité cross-générationnelle avec Qwen3 et surpasse les modèles Qwen3-VL sur les benchmarks de raisonnement, de code, d'agents et de compréhension visuelle.

- **Architecture Hybride Efficace** : Les Gated Delta Networks combinés à une Mixture-of-Experts parcimonieuse offrent une inférence à haut débit avec une latence et un coût minimaux.

- **Généralisation RL à l'échelle** : L'apprentissage par renforcement a été mis à l'échelle sur des environnements à plusieurs millions d'agents avec des distributions de tâches de complexité progressive, pour une adaptabilité robuste au monde réel.

- **Couverture linguistique mondiale** : Support étendu à 201 langues et dialectes, permettant un déploiement inclusif et mondial avec une compréhension culturelle et régionale fine.

- **Infrastructure d'entraînement de nouvelle génération** : Efficacité d'entraînement multimodal proche de 100 % comparée à l'entraînement texte uniquement, et frameworks RL asynchrones supportant des infrastructures d'agents massives et l'orchestration d'environnements. 

## Benchmarks

![benchmark](https://ollama.com/assets/library/qwen3.5/1c5d9a27-97b2-4d6d-a1b1-d326259acae5)

### Langue

<div style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;color:#1a1a2e;max-width:900px;margin:0 auto;padding:16px 0">
<table style="width:100%;border-collapse:collapse;font-size:13px">
<thead><tr>
<th style="padding:10px 12px;text-align:left;font-weight:600;border-bottom:2px solid #7c3aed;color:#4c1d95"></th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">GPT5.2</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">Claude 4.5 Opus</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">Gemini-3 Pro</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">Qwen3-Max-Thinking</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">K2.5-1T-A32B</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">Qwen3.5-397B-A17B</th>
</tr></thead>
<tbody>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Connaissances</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMLU-Pro</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">89.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">89.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.8</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMLU-Redux</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">95.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">95.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">95.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.9</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">SuperGPQA</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">74.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">69.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.4</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">C-Eval</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.0</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Suivi d'instructions</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">IFEval</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">IFBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">75.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">58.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.5</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MultiChallenge</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">57.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">54.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">64.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">63.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">62.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.6</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Contexte long</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">AA-LCR</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">72.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">74.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">LongBench v2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">54.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">64.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">60.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">61.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">63.2</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">STEM</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">GPQA</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">91.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.4</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">HLE</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">35.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">30.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">37.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">30.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">30.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">28.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">HLE-Verified¹</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">43.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">38.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">48</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">37.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">37.6</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Raisonnement</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">LiveCodeBench v6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">HMMT Feb 25</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">99.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">97.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">98.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">95.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.8</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">HMMT Nov 25</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">100</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">91.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">IMOAnswerBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.9</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">AIME26</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">96.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">91.3</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Agent généraliste</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">BFCL-V4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">63.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">72.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">72.9</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">TAU2-Bench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">91.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">VITA-Bench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">38.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">56.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">51.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">40.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">41.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">49.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">DeepPlanning</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">44.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">33.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">23.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">28.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">14.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">34.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">Tool Decathlon</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">43.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">43.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">36.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">18.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">27.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">38.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MCP-Mark</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">57.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">42.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">53.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">33.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">29.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">46.1</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Agent de recherche³</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">HLE w/ tool</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">45.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">43.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">45.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">49.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">50.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">48.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">BrowseComp</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">65.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">59.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">53.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--/74.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">69.0/78.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">BrowseComp-zh</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">62.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">66.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">60.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">WideSearch</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">57.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">72.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">74.0</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">Seal-0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">45.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">47.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">45.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">46.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">57.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">46.9</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Multilinguisme</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMMLU</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">89.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.5</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMLU-ProX</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">78.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">82.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">NOVA-63</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">54.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">56.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">56.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">54.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">56.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">59.1</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">INCLUDE</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">82.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">Global PIQA</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">91.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">89.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">89.8</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">PolyMATH</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">62.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">64.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">43.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">73.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">WMT24++</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">78.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">78.9</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MAXIFE</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">72.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.2</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Agent de code</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">SWE-bench Verified</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">75.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.2</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">SWE-bench Multilingual</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">72.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">65.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">66.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">73.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">69.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">SecCodeBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">62.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">57.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">61.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">Terminal Bench 2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">54.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">59.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">54.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">22.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">50.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">52.5</td>
</tr>
</tbody>
</table>

<p style="margin-top:12px;font-size:11px;color:#888">
* HLE-Verified: a verified and revised version of Humanity’s Last Exam (HLE), accompanied by a transparent, component-wise verification protocol and a fine-grained error taxonomy. We open-source the dataset at https://huggingface.co/datasets/skylenage/HLE-Verified.<br>
* TAU2-Bench: we follow the official setup except for the airline domain, where all models are evaluated by applying the fixes proposed in the Claude Opus 4.5 system card.<br>
* MCPMark: GitHub MCP server uses v0.30.3 from api.githubcopilot.com; Playwright tool responses are truncated at 32k tokens.<br>
* Search Agent: most search agents built on our model adopt a simple context-folding strategy(256k): once the cumulative Tool Response length reaches a preset threshold, earlier Tool Responses are pruned from the history to keep the context within limits.<br>
* BrowseComp: we tested two strategies, simple context-folding achieved a score of 69.0, while using the same discard-all strategy as DeepSeek-V3.2 and Kimi K2.5 achieved 78.6.<br>
* WideSearch: we use a 256k context window without any context management.<br>
* MMLU-ProX: we report the averaged accuracy on 29 languages.<br>
* WMT24++: a harder subset of WMT24 after difficulty labeling and rebalancing; we report the averaged scores on 55 languages using XCOMET-XXL.<br>
* MAXIFE: we report the accuracy on English + multilingual original prompts (totally 23 settings).<br>
* Les cellules vides (--) indiquent des scores non encore disponibles ou non applicables.<br>
</p>

</div>

### Vision-Langage

<div style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;color:#1a1a2e;max-width:900px;margin:0 auto;padding:16px 0">
<table style="width:100%;border-collapse:collapse;font-size:13px">
<thead><tr>
<th style="padding:10px 12px;text-align:left;font-weight:600;border-bottom:2px solid #7c3aed;color:#4c1d95"></th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">GPT5.2</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">Claude 4.5 Opus</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">Gemini-3 Pro</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">Qwen3-VL-235B-A22B</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">K2.5-1T-A32B</th>
<th style="padding:10px 12px;text-align:center;font-weight:500;border-bottom:2px solid #7c3aed;color:#4c1d95;font-size: 14px;">Qwen3.5-397B-A17B</th>
</tr></thead>
<tbody>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">STEM et énigmes</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMMU</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.0</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMMU-Pro</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">69.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">78.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.0</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MathVision</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">74.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">74.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">Mathvista(mini)</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">We-Math</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">74.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.9</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">DynaMath</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">82.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">ZEROBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">10</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">12</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">ZEROBench_sub</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">33.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">28.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">39.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">28.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">33.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">41.0</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">BabyVision</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">34.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">14.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">49.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">22.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">36.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">52.3/43.3</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">VQA général</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">RealWorldQA</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.9</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMStar</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">73.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">78.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.8</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">HallusionBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">65.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">64.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">66.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">69.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">71.4</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMBench<sub><small>EN-DEV-v1.1</small></sub></td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">89.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">89.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">SimpleVQA</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">55.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">65.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">73.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">61.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">71.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.1</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Reconnaissance de texte et compréhension de documents</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">OmniDocBench1.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.8</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">CharXiv(RQ)</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">82.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">66.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.8</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMLongBench-Doc</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">61.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">60.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">56.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">58.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">61.5</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">CC-OCR</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">82.0</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">AI2D_TEST</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">89.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.9</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">OCRBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.1</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Intelligence spatiale</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">ERQA</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">59.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">46.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">52.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.5</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">CountBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">91.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">90.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">97.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">93.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">94.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">97.2</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">RefCOCO(avg)</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">91.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">92.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">ODInW13</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">46.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">43.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">47.0</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">EmbSpatialBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">75.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">61.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.5</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">RefSpatialBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">65.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">69.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">73.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">LingoQA</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">78.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">72.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">66.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">68.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">V*</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">75.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">95.8/91.1</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">Hypersim</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">11.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">12.5</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">SUNRGBD</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">34.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">38.3</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">Nuscene</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">13.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">16.0</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Compréhension vidéo</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">VideoMME<sub><small>(w sub.)</sub></small></td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">88.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.5</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">VideoMME<sub><small>(w/o sub.)</sub></small></td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">VideoMMMU</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">87.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">84.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MLVU (M-Avg)</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">83.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">85.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">86.7</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MVBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">78.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">67.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">74.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">75.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">73.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">LVBench</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">73.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">57.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">63.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">75.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">75.5</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MMVU</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.8</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">77.5</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">71.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">80.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">75.4</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">Agent visuel</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">ScreenSpot Pro</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">45.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">72.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">62.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">65.6</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">OSWorld-Verified</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">38.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">66.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">38.1</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">63.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">62.2</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">AndroidWorld</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">63.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">--</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">66.8</td>
</tr>
<tr><td colspan="7" style="padding:8px 12px;font-weight:600;color:#7c3aed;border-bottom:1px solid #e5e7eb;background:#faf5ff">VQA médical</td></tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">SLAKE</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.4</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">54.7</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">81.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">79.9</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">PMC-VQA</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">58.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">59.9</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">62.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">41.2</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">63.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">64.2</td>
</tr>
<tr>
<td style="padding:7px 12px;padding-left:20px;border-bottom:1px solid #f0f0f0;color:#444">MedXpertQA-MM</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">73.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">63.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">76.0</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">47.6</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">65.3</td>
<td style="padding:7px 12px;text-align:center;border-bottom:1px solid #f0f0f0">70.0</td>
</tr>
</tbody>
</table>

<p style="margin-top:12px;font-size:11px;color:#888">
* MathVision : le score de notre modèle est évalué avec un prompt fixe, par exemple « Please reason step by step, and put your final answer within \boxed{}. » Pour les autres modèles, nous rapportons le score le plus élevé entre les runs avec et sans le formatage \boxed{}.<br>
* BabyVision : le score de notre modèle est rapporté avec CI (interpréteur de code) activé ; sans CI, le résultat est 43.3.<br>
* V* : le score de notre modèle est rapporté avec CI (interpréteur de code) activé ; sans CI, le résultat est 91.1.<br>
* Les cellules vides (--) indiquent des scores non encore disponibles ou non applicables.<br>
</p>

</div>