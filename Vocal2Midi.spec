# -*- mode: python ; coding: utf-8 -*-
from PyInstaller.utils.hooks import collect_submodules
from PyInstaller.utils.hooks import collect_all

datas = []
binaries = []
hiddenimports = ['onnxruntime', 'onnxruntime.capi', 'onnxruntime.capi.onnxruntime_pybind11_state', 'onnxruntime.transformers', 'PyQt5.QtSvg', 'PyQt5.QtPrintSupport', 'PyQt5.QtWebEngineWidgets', 'librosa', 'soundfile', 'scipy.signal', 'yaml', 'click', 'textgrid', 'mido', 'numpy']
hiddenimports += collect_submodules('gui')
hiddenimports += collect_submodules('inference')
hiddenimports += collect_submodules('application')
hiddenimports += collect_submodules('scripts')
tmp_ret = collect_all('PyQt5')
datas += tmp_ret[0]; binaries += tmp_ret[1]; hiddenimports += tmp_ret[2]
tmp_ret = collect_all('PyQt-Fluent-Widgets')
datas += tmp_ret[0]; binaries += tmp_ret[1]; hiddenimports += tmp_ret[2]
tmp_ret = collect_all('onnxruntime')
datas += tmp_ret[0]; binaries += tmp_ret[1]; hiddenimports += tmp_ret[2]


a = Analysis(
    ['app_fluent.py'],
    pathex=['/home/fuurin/code/Vocal2Midi-for-linux'],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='Vocal2Midi',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='Vocal2Midi',
)
