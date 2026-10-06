"""Plot actual native checks; distinguish observations outside the window."""
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

P = Path(__file__).resolve().parent
data = json.loads((P / 'analysis-normal.json').read_text())
fig, axes = plt.subplots(1, 2, figsize=(14, 5), layout='constrained')
for index, row in enumerate(data['rows']):
    label = f"{row['kind']} / {row['load']} / h={row['h']:g}"
    color = plt.get_cmap('tab10')(index)
    y = [x / 1e-13 for x in row['actual_check_rate_norms']]
    axes[0].plot(range(1, 8), y, 'o-', color=color, label=label)
    axes[1].plot(range(3, 8), y[2:], 'o-', color=color)
    axes[1].plot(8, row['actual_unused_seventh_correction_rate_16'] / 1e-13,
                 marker='D', markerfacecolor='none', markeredgecolor=color, markersize=8)
axes[0].set_yscale('log')
axes[0].set_title('All seven actual native residual checks')
axes[1].set_title('Late checks; hollow diamonds are outside-window observations')
for ax in axes:
    ax.axhline(1, color='firebrick', linestyle='--', label='unchanged Newton threshold')
    ax.set_xlabel('Residual check index (8 is observational only)')
    ax.set_ylabel('Native rate norm / 1e-13')
    ax.grid(alpha=.25)
axes[0].legend(fontsize=8)
axes[1].set_xticks(range(3, 9))
fig.suptitle('All six original steps remain refused; no observation publishes or replaces the failed convergence criterion', fontsize=11)
fig.savefig(P / 'newton-window.png', dpi=180)
fig.savefig(P / 'newton-window.pdf')
