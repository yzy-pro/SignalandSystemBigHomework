#!/bin/bash
# 导出频谱分析结果为 CSV 文件

echo "=========================================="
echo "Q1: 导出频谱分析结果为 CSV"
echo "=========================================="
echo ""

# 编译并运行 export_csv 程序
echo "正在编译并运行导出程序..."
cargo run --release --bin export_csv

# 检查是否成功
if [ $? -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "导出成功！"
    echo "=========================================="
    echo ""
    echo "生成的 CSV 文件："
    ls -lh output/*.csv
    echo ""
    echo "CSV 文件说明："
    echo "  - Q1_spectrum_full.csv: 完整频谱数据（所有频率）"
    echo "  - Q1_spectrum_lowfreq.csv: 低频段频谱 (0-4 kHz)"
    echo "  - Q1_spectrum_midfreq.csv: 中频段频谱 (0-10 kHz)"
    echo "  - Q1_waveform.csv: 时域波形数据（前10000个采样点）"
    echo "  - Q1_peaks.csv: 检测到的峰值列表"
    echo "  - Q1_energy_distribution.csv: 能量分布统计"
    echo "  - Q1_summary.csv: 分析结果摘要"
    echo ""
else
    echo ""
    echo "导出失败！请检查错误信息。"
    exit 1
fi
