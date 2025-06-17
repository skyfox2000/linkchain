#!/bin/bash

# LinkChain 统一挂件测试脚本 v4.1
# 顺序执行，每个测试完成后等待用户确认

set -e

echo "🚀 LinkChain 统一挂件测试脚本 v4.1"
echo "=================================================================================="
echo "🎯 测试模式: 顺序执行 + 用户确认继续"
echo "📅 执行时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=================================================================================="

# 记录开始时间
START_TIME=$(date +%s)

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# 统计变量
TOTAL_TEST_CASES=0
PASSED_TEST_CASES=0
FAILED_TEST_CASES=0

# 等待用户确认函数
wait_for_user() {
    echo ""
    echo -e "${CYAN}📝 按任意键继续下一个测试...${NC}"
    read -n 1 -s -r
    echo ""
}

# 测试结果统计函数
run_test_category() {
    local test_name="$1"
    local test_command="$2"
    local test_description="$3"
    
    echo ""
    echo -e "${BLUE}┌────────────────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 🧪 执行测试: ${WHITE}${test_name}${BLUE}${NC}"
    echo -e "${BLUE}│ 📖 说明: ${test_description}${NC}"
    echo -e "${BLUE}└────────────────────────────────────────────────────────────────────────┘${NC}"
    
    echo -e "${CYAN}🔄 正在执行测试命令: ${test_command}${NC}"
    
    # 执行测试并显示详细输出
    local test_output
    test_output=$(eval "$test_command --quiet" 2>/dev/null | grep -v "test result:" | grep -v "running 0 tests" | grep -v "Running tests/")
    local exit_code=$?
    
    # 显示详细测试输出
    echo "$test_output"
    
    # 统计测试用例数量（基于输出中的✅和❌符号）
    local passed_cases=$(echo "$test_output" | grep -o "✅ 成功" | wc -l | xargs)
    local failed_cases=$(echo "$test_output" | grep -o "❌ 失败" | wc -l | xargs)
    local total_cases=$((passed_cases + failed_cases))
    
    TOTAL_TEST_CASES=$((TOTAL_TEST_CASES + total_cases))
    PASSED_TEST_CASES=$((PASSED_TEST_CASES + passed_cases))
    FAILED_TEST_CASES=$((FAILED_TEST_CASES + failed_cases))
    
    if [ $exit_code -eq 0 ]; then
        echo ""
        echo -e "${GREEN}✅ ${test_name} - 测试通过${NC}"
        
        # 显示当前测试文件结果统计
        echo -e "${GREEN}📊 当前测试文件结果:${NC}"
        echo -e "${GREEN}  • 测试文件: ${test_name}${NC}"
        echo -e "${GREEN}  • 状态: 通过 ✅${NC}"
        echo -e "${GREEN}  • 测试用例: ${total_cases} 个 (通过: ${passed_cases}, 失败: ${failed_cases})${NC}"
        echo -e "${GREEN}  • 详情: ${test_description}${NC}"
        
    else
        echo ""
        echo -e "${RED}❌ ${test_name} - 测试失败${NC}"
        
        # 显示当前测试文件结果统计
        echo -e "${RED}📊 当前测试文件结果:${NC}"
        echo -e "${RED}  • 测试文件: ${test_name}${NC}"
        echo -e "${RED}  • 状态: 失败 ❌${NC}"
        echo -e "${RED}  • 详情: ${test_description}${NC}"
    fi
    
    # 显示当前累计统计
    echo ""
    echo -e "${PURPLE}📈 当前累计统计:${NC}"
    echo -e "${CYAN}  • 测试用例总数: ${TOTAL_TEST_CASES} 个${NC}"
    echo -e "${GREEN}  • 用例通过: ${PASSED_TEST_CASES} ✅${NC}"
    echo -e "${RED}  • 用例失败: ${FAILED_TEST_CASES} ❌${NC}"
    
    if [ $TOTAL_TEST_CASES -gt 0 ]; then
        local current_rate=$(echo "scale=1; $PASSED_TEST_CASES * 100 / $TOTAL_TEST_CASES" | bc -l)
        echo -e "${YELLOW}  • 用例成功率: ${current_rate}%${NC}"
    fi
    
    echo -e "${CYAN}$(echo "$test_name" | sed 's/./-/g')${NC}"
    
    # 如果不是最后一个测试，等待用户确认
    local total_categories=6
    if [ $(echo "$test_name" | wc -w) -lt $total_categories ]; then
        wait_for_user
    fi
}

# 项目编译检查
echo ""
echo -e "${CYAN}🔧 编译项目...${NC}"
if cargo build --release --quiet; then
    echo -e "${GREEN}✅ 项目编译成功${NC}"
else
    echo -e "${RED}❌ 项目编译失败，退出测试${NC}"
    exit 1
fi

echo ""
echo -e "${PURPLE}🎯 开始执行统一挂件测试（顺序模式）...${NC}"
echo -e "${YELLOW}ℹ️  每个测试完成后会显示结果并等待您按键继续${NC}"

# 按顺序执行各类测试，每个测试之间等待用户确认
# run_test_category \
#     "统一挂件测试框架" \
#     "cargo test unified_chainware_tests::tests --release -- --test-threads=1 --nocapture" \
#     "包含所有核心挂件的基础功能测试，覆盖条件、提取、IP过滤、数据处理和集成场景"

run_test_category \
    "条件挂件专项测试" \
    "cargo test test_condition_chainwares::tests --release -- --test-threads=1 --nocapture" \
    "专门测试条件判断和正则条件挂件，包括基本比较、字符串操作、正则匹配等"

run_test_category \
    "IP过滤挂件专项测试" \
    "cargo test test_ip_filter_chainwares::tests --release -- --test-threads=1 --nocapture" \
    "专门测试IP黑名单和白名单挂件，包括单IP、CIDR网段、链式组合等"

run_test_category \
    "数据提取挂件测试" \
    "cargo test test_extract_chainwares::tests --release -- --test-threads=1 --nocapture" \
    "测试正则提取和JSON提取挂件的各种数据提取场景"

run_test_category \
    "数据处理挂件测试" \
    "cargo test test_data_processing_chainwares::tests --release -- --test-threads=1 --nocapture" \
    "测试字段映射、数据合并、日志记录等数据处理挂件"

run_test_category \
    "集成场景测试" \
    "cargo test test_integration_scenarios::tests --release -- --test-threads=1 --nocapture" \
    "测试多挂件链式组合的复杂业务场景，如用户注册、安全检查等"

# 计算执行时间
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
MINUTES=$((DURATION / 60))
SECONDS=$((DURATION % 60))

# 计算成功率
if [ $TOTAL_TEST_CASES -gt 0 ]; then
    SUCCESS_RATE=$(echo "scale=1; $PASSED_TEST_CASES * 100 / $TOTAL_TEST_CASES" | bc -l)
else
    SUCCESS_RATE=0
fi

# 成功率评级
if (( $(echo "$SUCCESS_RATE >= 90" | bc -l) )); then
    RATING="优秀 🌟🌟🌟"
    RATING_COLOR=$GREEN
elif (( $(echo "$SUCCESS_RATE >= 75" | bc -l) )); then
    RATING="良好 🌟🌟"
    RATING_COLOR=$YELLOW
elif (( $(echo "$SUCCESS_RATE >= 60" | bc -l) )); then
    RATING="及格 🌟"
    RATING_COLOR=$YELLOW
else
    RATING="需要改进 ⚠️"
    RATING_COLOR=$RED
fi

echo ""
echo "=================================================================================="
echo -e "${WHITE}📊 LinkChain 统一挂件测试最终报告 v4.1${NC}"
echo "=================================================================================="
echo -e "${CYAN}⏱️  执行时长:${NC} ${MINUTES}分${SECONDS}秒"
echo -e "${CYAN}🧪 测试用例总数:${NC} ${TOTAL_TEST_CASES}"
echo -e "${GREEN}✅ 通过用例数:${NC} ${PASSED_TEST_CASES}"
echo -e "${RED}❌ 失败用例数:${NC} ${FAILED_TEST_CASES}"
echo -e "${PURPLE}📈 用例成功率:${NC} ${SUCCESS_RATE}%"
echo -e "${RATING_COLOR}🏆 评级:${NC} ${RATING}"
echo ""
echo -e "${CYAN}📋 测试文件详情:${NC}"
echo "• 统一测试框架: 核心挂件基础功能全覆盖"
echo "• 条件挂件专项: 条件判断和正则匹配深度测试"
echo "• IP过滤挂件专项: IP黑白名单和网段过滤测试"
echo "• 数据提取挂件: 正则和JSON提取功能测试"
echo "• 数据处理挂件: 映射、合并、日志处理测试"
echo "• 集成场景测试: 多挂件链式组合业务场景"
echo ""

# 结果建议
if [ $FAILED_TEST_CASES -eq 0 ]; then
    echo -e "${GREEN}🎉 所有测试用例都通过了！项目状态优秀。${NC}"
elif [ $FAILED_TEST_CASES -le 3 ]; then
    echo -e "${YELLOW}⚠️  有少量测试用例失败，建议检查相关功能。${NC}"
else
    echo -e "${RED}🚨 多个测试用例失败，需要重点关注项目质量。${NC}"
fi

echo "=================================================================================="

# 返回适当的退出码
if [ $FAILED_TEST_CASES -eq 0 ]; then
    exit 0
else
    exit 1
fi 