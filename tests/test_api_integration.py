#!/usr/bin/env python3
"""
Vocal2Midi Web API - 完整集成测试脚本
使用真实音频文件测试所有 REST API 和 WebSocket 功能

运行方式:
    python tests/test_api_integration.py
"""

import requests
import json
import time
import sys
from pathlib import Path

# API 基础地址
BASE_URL = "http://localhost:5000"
API_BASE = f"{BASE_URL}/api"

# 测试文件路径
TEST_DIR = Path(__file__).parent
ZH_AUDIO = TEST_DIR / "zh-bpm-98.flac"
JP_AUDIO = TEST_DIR / "jp-bpm-126.flac"

# 全局变量存储 task_id
current_task_id = None


def print_section(title):
    """打印测试章节标题"""
    print(f"\n{'='*60}")
    print(f"🔍 {title}")
    print('='*60)


def print_test(name, result, details=""):
    """打印单个测试结果"""
    status = "✅ PASS" if result else "❌ FAIL"
    icon = "✅" if result else "❌"
    print(f"  {icon} {name}")
    if details:
        print(f"     → {details}")


def test_api(endpoint, method="GET", data=None, files=None, expected_status=200):
    """
    通用 API 测试函数
    
    Args:
        endpoint: API 端点 (如 /system/info)
        method: HTTP 方法 (GET/POST/PUT)
        data: JSON 数据 (dict)
        files: 文件数据 (dict)
        expected_status: 期望的 HTTP 状态码
    
    Returns:
        (success: bool, response_data: dict, status_code: int)
    """
    url = f"{API_BASE}{endpoint}"
    
    try:
        if method == "GET":
            resp = requests.get(url, timeout=10)
        elif method == "POST":
            if files:
                resp = requests.post(url, files=files, data=data, timeout=30)
            else:
                resp = requests.post(url, json=data, timeout=10)
        elif method == "PUT":
            resp = requests.put(url, json=data, timeout=10)
        
        try:
            resp_data = resp.json()
        except:
            resp_data = {"raw": resp.text}
        
        success = (resp.status_code == expected_status)
        return success, resp_data, resp.status_code
        
    except Exception as e:
        return False, {"error": str(e)}, 0


# ==================== 测试函数 ====================

def test_1_system_info():
    """测试系统信息 API"""
    print_section("1️⃣  系统信息 API (GET /api/system/info)")
    
    success, data, status = test_api("/system/info")
    
    checks = [
        ("HTTP 状态码 200", status == 200),
        ("返回 success=True", data.get("success") == True),
        ("包含 version 字段", "version" in data),
        ("包含 python_version", "python_version" in data),
        ("包含 device 信息", "device" in data),
        ("包含 available_devices 列表", isinstance(data.get("available_devices"), list)),
    ]
    
    all_pass = True
    for name, result in checks:
        print_test(name, result)
        all_pass = all_pass and result
    
    if status == 200:
        print(f"\n  📊 系统信息:")
        print(f"     版本: {data.get('version')}")
        print(f"     Python: {data.get('python_version')}")
        print(f"     平台: {data.get('platform')}")
        print(f"     设备: {data.get('device')}")
        print(f"     可用设备: {', '.join(data.get('available_devices', []))}")
        print(f"     活跃任务数: {data.get('active_tasks', 0)}")
    
    return all_pass


def test_2_settings_get():
    """测试获取设置 API"""
    print_section("2️⃣  获取设置 API (GET /api/settings)")
    
    success, data, status = test_api("/settings")
    
    checks = [
        ("HTTP 状态码 200", status == 200),
        ("返回 success=True", data.get("success") == True),
        ("包含 models 配置", "models" in data),
        ("包含 params 参数", "params" in data),
        ("包含 debug 选项", "debug" in data),
        ("GAME 模型路径不为空", bool(data.get("models", {}).get("game_model_path"))),
        ("seg_threshold 有默认值", "seg_threshold" in data.get("params", {})),
    ]
    
    all_pass = True
    for name, result in checks:
        print_test(name, result)
        all_pass = all_pass and result
    
    return all_pass


def test_3_settings_update_and_reset():
    """测试更新和重置设置 API"""
    print_section("3️⃣  更新与重置设置 API (PUT/POST /api/settings)")
    
    # 3.1 更新设置
    print("\n  📝 测试更新设置...")
    update_data = {
        "params": {
            "seg_threshold": 0.99,
            "slice_min": 5.0,
            "nsteps": 15
        },
        "debug": {
            "export_txt": True,
            "round_pitch": False
        }
    }
    
    success, data, status = test_api("/settings", "PUT", update_data)
    
    check1 = ("更新设置 HTTP 200", status == 200 and data.get("success"))
    print_test(*check1)
    
    # 3.2 验证更新生效
    success2, data2, _ = test_api("/settings")
    seg_val = data2.get("params", {}).get("seg_threshold")
    check2 = (f"seg_threshold 已更新为 0.99", seg_val == 0.99)
    print_test(*check2)
    
    # 3.3 重置设置
    print("\n  🔄 测试重置设置...")
    success3, data3, status3 = test_api("/settings/reset", "POST")
    
    check3 = ("重置设置 HTTP 200", status3 == 200 and data3.get("success"))
    print_test(*check3)
    
    # 3.4 验证重置生效
    success4, data4, _ = test_api("/settings")
    reset_seg = data4.get("params", {}).get("seg_threshold")
    check4 = (f"seg_threshold 已恢复默认值 0.2", reset_seg == 0.2)
    print_test(*check4)
    
    all_pass = all([check1[1], check2[1], check3[1], check4[1]])
    return all_pass


def test_4_upload_and_start_task():
    """测试文件上传和任务启动"""
    global current_task_id
    
    print_section("4️⃣  文件上传 + 任务启动 (POST /api/pipeline/start)")
    
    # 检查测试文件是否存在
    if not ZH_AUDIO.exists():
        print_test("测试音频文件存在", False, f"文件不存在: {ZH_AUDIO}")
        return False
    
    print(f"\n  🎵 使用测试文件: {ZH_AUDIO.name} ({ZH_AUDIO.stat().st_size / 1024:.1f} KB)")
    
    # 准备配置
    config = json.dumps({
        "language": "zh",
        "device": "cpu",
        "tempo": 98,
        "save_dir": "./test_output_api",
        "enable_lyrics_match": False,
        "output_lyrics": True
    })
    
    # 上传文件并启动任务
    files = {'audio_file': open(ZH_AUDIO, 'rb')}
    form_data = {'config': config}
    
    url = f"{API_BASE}/pipeline/start"
    try:
        resp = requests.post(url, files=files, data=form_data, timeout=30)
        files['audio_file'].close()  # 关闭文件句柄
        
        data = resp.json()
        
        checks = [
            ("HTTP 状态码 200", resp.status_code == 200),
            ("返回 success=True", data.get("success") == True),
            ("包含 task_id", "task_id" in data),
            ("状态为 running", data.get("status") == "running"),
            ("task_id 格式正确 (UUID)", len(data.get("task_id", "")) > 30),
        ]
        
        all_pass = True
        for name, result in checks:
            print_test(name, result)
            all_pass = all_pass and result
        
        if all_pass:
            current_task_id = data['task_id']
            print(f"\n  ✨ 任务已启动!")
            print(f"     Task ID: {current_task_id[:8]}...{current_task_id[-4:]}")
            print(f"     完整 ID: {current_task_id}")
        
        return all_pass
        
    except Exception as e:
        print_test("请求成功", False, str(e))
        return False


def test_5_task_status_query():
    """测试任务状态查询"""
    global current_task_id
    
    if not current_task_id:
        print_section("5️⃣  任务状态查询 (跳过 - 无活跃任务)")
        return True
    
    print_section("5️⃣  任务状态查询 (GET /api/pipeline/status/<id>)")
    
    time.sleep(0.5)  # 等待一小段时间让任务初始化
    
    success, data, status = test_api(f"/pipeline/status/{current_task_id}")
    
    valid_statuses = ['pending', 'running', 'completed', 'failed', 'cancelled']
    
    checks = [
        ("HTTP 状态码 200", status == 200),
        ("返回 success=True", data.get("success") == True),
        ("task_id 匹配", data.get("task_id") == current_task_id),
        ("status 有效", data.get("status") in valid_statuses),
        ("progress 是整数", isinstance(data.get("progress"), int)),
        ("progress 范围 0-100", 0 <= data.get("progress", -1) <= 100),
        ("stage 不为空", bool(data.get("stage"))),
        ("包含 created_at 时间戳", "created_at" in data),
    ]
    
    all_pass = True
    for name, result in checks:
        print_test(name, result)
        all_pass = all_pass and result
    
    if status == 200:
        print(f"\n  📊 当前任务状态:")
        print(f"     状态: {data.get('status')}")
        print(f"     进度: {data.get('progress')}%")
        print(f"     阶段: {data.get('stage')}")
        print(f"     创建时间: {data.get('created_at')}")
        print(f"     开始时间: {data.get('started_at', '未开始')}")
        if data.get('error'):
            print(f"     错误: {data.get('error')}")
    
    return all_pass


def test_6_list_tasks():
    """测试列出所有任务"""
    print_section("6️⃣  列出所有任务 (GET /api/pipeline/list)")
    
    success, data, status = test_api("/pipeline/list")
    
    checks = [
        ("HTTP 状态码 200", status == 200),
        ("返回 success=True", data.get("success") == True),
        ("包含 tasks 列表", "tasks" in data),
        ("包含 count 计数", "count" in data),
        ("count 与列表长度一致", data.get('count', -1) == len(data.get('tasks', []))),
        ("至少有 1 个任务（刚才创建的）", len(data.get('tasks', [])) >= 1),
    ]
    
    all_pass = True
    for name, result in checks:
        print_test(name, result)
        all_pass = all_pass and result
    
    if status == 200:
        print(f"\n  📋 当前任务总数: {data.get('count')}")
        for i, task in enumerate(data.get('tasks', [])[:3]):  # 只显示前3个
            print(f"     [{i+1}] {task.get('task_id', '?')[:12]}... | "
                  f"{task.get('status')} | "
                  f"{task.get('progress')}% | "
                  f"{task.get('stage')}")
    
    return all_pass


def test_7_stop_nonexistent_task():
    """测试停止不存在的任务"""
    print_section("7️⃣  停止不存在的任务 (POST /api/pipeline/stop)")
    
    fake_id = "00000000-0000-0000-0000-000000000000"
    success, data, status = test_api("/pipeline/stop", "POST", {"task_id": fake_id}, expected_status=404)
    
    checks = [
        ("HTTP 状态码 404", status == 404),
        ("返回 success=False", data.get("success") == False),
        ("包含错误信息", "error" in data),
    ]
    
    all_pass = True
    for name, result in checks:
        print_test(name, result)
        all_pass = all_pass and result
    
    return all_pass


def test_8_error_handling():
    """测试错误处理"""
    print_section("8️⃣  错误处理测试")
    
    results = []
    
    # 8.1 无效路由
    print("\n  🔗 测试无效路由...")
    success, _, status = test_api("/invalid/endpoint", expected_status=404)
    results.append(("无效路由返回 404", success))
    
    # 8.2 无效 JSON (使用 requests 直接发送以避免自动编码)
    print("\n  📄 测试无效 JSON...")
    try:
        url = f"{API_BASE}/settings"
        resp = requests.put(url, data="not-valid-json-string", 
                          headers={"Content-Type": "application/json"}, timeout=5)
        success = (resp.status_code == 400)
        data = resp.json()
        has_error_msg = ("error" in data and "invalid" in str(data.get("error", "")).lower())
        results.append(("无效 JSON 返回 400", success and has_error_msg))
        if not success:
            print(f"     → 实际状态码: {resp.status_code}, 响应: {data}")
    except Exception as e:
        results.append(("无效 JSON 返回 400", False, str(e)))
    
    # 8.3 无文件上传启动
    print("\n  📤 测试无文件上传...")
    success, data, status = test_api("/pipeline/start", "POST", {}, expected_status=400)
    results.append(("无文件上传返回 400", success and 'audio file' in str(data).lower()))
    
    all_pass = all(r[1] for r in results)
    for name, result in results:
        print_test(name, result)
    
    return all_pass


def test_9_websocket_connection():
    """测试 WebSocket 连接（基础）"""
    print_section("9️⃣  WebSocket 连接测试")
    
    try:
        import socketio
        print("\n  🔌 尝试连接 WebSocket...")
        
        sio = socketio.Client()
        
        # 定义事件处理器
        events_received = []
        
        @sio.on('connect')
        def on_connect():
            events_received.append('connect')
            print("  ✅ WebSocket 连接成功!")
        
        @sio.on('disconnect')
        def on_disconnect():
            events_received.append('disconnect')
        
        @sio.on('connected')
        def on_connected(data):
            events_received.append('connected')
            print(f"  📨 收到 connected 事件: {data}")
        
        # 连接到服务器
        sio.connect(BASE_URL, transports=['websocket'], wait_timeout=5)
        
        time.sleep(0.5)  # 等待连接建立
        
        # 检查连接状态
        checks = [
            ("SocketIO 连接成功", sio.connected),
            ("收到 connect 事件", 'connect' in events_received),
            ("收到 connected 服务端确认", 'connected' in events_received),
        ]
        
        all_pass = True
        for name, result in checks:
            print_test(name, result)
            all_pass = all_pass and result
        
        # 断开连接
        sio.disconnect()
        print("\n  🔌 WebSocket 连接已断开")
        
        return all_pass
        
    except ImportError:
        print_test("python-socketio 库已安装", False, "请运行: pip install python-socketio")
        return False
    except Exception as e:
        print_test("WebSocket 连接成功", False, str(e))
        return False


# ==================== 主测试流程 ====================

def main():
    """运行所有测试"""
    print("\n" + "="*70)
    print("  🚀 Vocal2Midi Web API - 完整集成测试")
    print("="*70)
    print(f"  📍 服务器地址: {BASE_URL}")
    print(f"  📁  测试目录: {TEST_DIR}")
    print(f"  🎵  中文音频: {ZH_AUDIO.name} ({'✅ 存在' if ZH_AUDIO.exists() else '❌ 缺失'})")
    print(f"  🎵  日文音频: {JP_AUDIO.name} ({'✅ 存在' if JP_AUDIO.exists() else '❌ 缺失'})")
    print("="*70)
    
    # 首先检查服务器是否运行
    print("\n🔄 检查服务器状态...")
    try:
        resp = requests.get(BASE_URL, timeout=5)
        if resp.status_code == 200:
            print("  ✅ 服务器正常运行")
        else:
            print(f"  ❌ 服务器返回异常: {resp.status_code}")
            print("\n  ⚠️  请先启动服务器:")
            print("     python web_server.py")
            return False
    except requests.exceptions.ConnectionError:
        print("  ❌ 无法连接到服务器!")
        print("\n  ⚠️  请先启动 Web 服务器:")
        print("     cd /home/fuurin/code/Vocal2Midi-for-linux")
        print("     python web_server.py")
        return False
    
    # 运行所有测试
    start_time = time.time()
    
    results = []
    
    # 执行测试
    results.append(("系统信息 API", test_1_system_info()))
    results.append(("获取设置 API", test_2_settings_get()))
    results.append(("更新/重置设置 API", test_3_settings_update_and_reset()))
    results.append(("文件上传+启动任务", test_4_upload_and_start_task()))
    results.append(("任务状态查询", test_5_task_status_query()))
    results.append(("列出所有任务", test_6_list_tasks()))
    results.append(("停止不存在任务", test_7_stop_nonexistent_task()))
    results.append(("错误处理", test_8_error_handling()))
    results.append(("WebSocket 连接", test_9_websocket_connection()))
    
    # 统计结果
    elapsed = time.time() - start_time
    total = len(results)
    passed = sum(1 for _, r in results if r)
    failed = total - passed
    pass_rate = (passed / total * 100) if total > 0 else 0
    
    # 打印总结
    print("\n" + "="*70)
    print("  📊 测试结果总结")
    print("="*70)
    
    for name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"  {status}  {name}")
    
    print("-"*70)
    print(f"  总计: {total} 个测试套件")
    print(f"  通过: {passed} 个 ({pass_rate:.1f}%)")
    print(f"  失败: {failed} 个")
    print(f"  耗时: {elapsed:.2f} 秒")
    print("="*70)
    
    if failed == 0:
        print("\n  🎉 所有测试通过！Vocal2Midi Web API 运行正常！\n")
        return True
    else:
        print(f"\n  ⚠️  有 {failed} 个测试失败，请检查上方详情。\n")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
