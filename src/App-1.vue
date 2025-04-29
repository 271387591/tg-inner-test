<script setup>
import { ref, onMounted, onUnmounted, nextTick, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl,openPath } from '@tauri-apps/plugin-opener';
import {open, save} from '@tauri-apps/plugin-dialog';
import { exit, relaunch } from '@tauri-apps/plugin-process';
import Swal from "sweetalert2";
// 导入logo
import logo from './assets/logo.svg';
const status = ref(false);
const logs = ref([]);  // 初始化为空数组，不显示初始消息
const error = ref("");  // 添加错误状态
const hasUpdate = ref(false); // 临时设置为true用于测试
const currentVersion = ref(""); // 设置当前版本
const latestVersion = ref(); // 设置最新版本
const latestUrl = ref(); // 设置最新版本
const update_file_path = ref(); // 设置最新版本
const versionDetail = ref({}); // 设置最新版本
const os_detail = ref(''); // 是否正在手动滚动
const MAX_LOGS = 50; // 最大日志条数
const frame_url = ref(''); // 控制是否显示日志区域
const iframeLoaded = ref(false); // 控制iframe加载状态
const showLogPopup = ref(false); // 控制日志弹窗显示
const core_current_version = ref(''); // 控制日志弹窗显示
const core_last_version = ref(''); // 控制日志弹窗显示
const core_has_update = ref(false); // 控制日志弹窗显示
const coreLastUrl = ref(); // 控制日志弹窗显示
const iframeRetryCount = ref(0); // iframe加载重试次数
const MAX_RETRY_COUNT = 5; // 最大重试次数
const isRetrying = ref(false); // 是否正在重试中
const retryTimeoutId = ref(null); // 重试计时器ID
const loadTimeoutId = ref(null); // 加载超时计时器ID
const IFRAME_LOAD_TIMEOUT = 10000; // iframe加载超时时间（毫秒）

const showLoading = (title,msg) => {
  Swal.fire({
    title: title,
    html: `<b>${msg}</b>`,
    allowEscapeKey: false,
    allowOutsideClick: false,
    timerProgressBar: true,
    didOpen: () => {
      Swal.showLoading();
    }
  })
}
const showError = async (title,msg) => {
  return Swal.fire({
    title: title,
    text: msg,
    icon: 'error'
  });
}

const do_exit = async () => {
  const result = await Swal.fire({
    title: '确认退出',
    text: `你确定要退出程序？`,
    icon: 'info',
    showCancelButton: true,
    confirmButtonText: '退出程序',
    cancelButtonText: '取消'
  });
  // 如果用户点击了取消按钮，直接返回
  if (!result.isConfirmed) {
    return;
  }
  Swal.close()
  await invoke("tg_exit")
}


const setupListeners = async () => {
  await listen("js_event", (event) => {
    if (!event.payload || event.payload.startsWith("#")) return;
    logs.value.push(event.payload);
    // 限制最大日志条数，超出时移除最早的日志
    if (logs.value.length > MAX_LOGS) {
      logs.value = logs.value.slice(logs.value.length - MAX_LOGS);
    }
  });
  await listen("js_status", (event) => {
    console.log("js_status",event);
    Swal.close()
    if (event.payload.status===1){
      status.value = true;
      frame_url.value = event.payload.url;
      nextTick(()=>{
        console.log("status nextTick",frame_url.value);
        setTimeout(()=>{
          location.replace(frame_url.value);
        },3000)

        // const iframe = document.querySelector('iframe');
        // iframe.src = `${frame_url.value}`;
      })
      console.log("frame_url.value",frame_url.value)
      get_version_info()
      
      // 设置iframe加载超时检测
      startIframeLoadTimeout();
      
      return;
    }
    Swal.fire({
      title: "退出程序",
      text: "无法启动程序，请联系管理员！",
      icon: "error",
      allowOutsideClick: false,
      showCancelButton: false,
      confirmButtonColor: "#d33",
      confirmButtonText: "退出程序"
    }).then((result) => {
      if (result.isConfirmed) {
        invoke("tg_exit")
      }
    });
  });
  await listen("js_exit", async (event) => {
    Swal.close()
    if (event.payload=="0"){
      showLoading('关闭中......',"系统正在关闭")
    }else if(event.payload=="1"){
      await exit(0);
    }
  });
  await listen("js_update", async (event) => {
    let payload=event.payload;
    let step=payload.step;
    if (step===0){
      // 显示下载进度对话框
      const  progressDialog = await Swal.fire({
        title: '正在下载更新',
        html: `
        <div class="w-full bg-gray-200 rounded-full h-2.5">
          <div class="bg-blue-600 h-2.5 rounded-full" style="width: 0%"></div>
        </div>
        <div class="mt-2 text-sm text-gray-600">0%</div>
        <div class="mt-2 text-xs text-gray-500">
          已下载: ${formatFileSize(0)} / ${formatFileSize(0)}
        </div>
      `,
        showConfirmButton: false,
        allowOutsideClick: false,
        didOpen: () => {
          Swal.showLoading();
        },
        willClose: () => {
        }
      });
      // 等待用户点击"加载新版本"按钮
      if (progressDialog.isConfirmed) {
        console.log("点击确认更新...")
        // 这里添加更新逻辑
        let rs = await invoke("do_perform_update",{updateFilePath:update_file_path.value});
        if (!rs){
          await showError('更新失败','更新过程中发生错误')
          return
        }
        Swal.close();
        showLoading('更新中......',"系统正在更新")
      }
    }else if(step===1){
      update_file_path.value=payload.file_path
      const progressBar = Swal.getPopup().querySelector('.bg-blue-600');
      const progressText = Swal.getPopup().querySelector('.text-gray-600');
      const sizeText = Swal.getPopup().querySelector('.text-gray-500');
      if (payload.percentage===100){
        Swal.update({
          title: '下载完成',
          html: `
                <div class="w-full bg-gray-200 rounded-full h-2.5">
                  <div class="bg-green-600 h-2.5 rounded-full" style="width: 100%"></div>
                </div>
                <div class="mt-2 text-sm text-gray-600">100%</div>
                <div class="mt-2 text-xs text-gray-500">
                  已下载: ${formatFileSize(payload.downloaded)} / ${formatFileSize(payload.total)}
                </div>
              `,
          showConfirmButton: true,
          confirmButtonText: '加载新版本'
        });
        return
      }
      progressBar.style.width = `${payload.percentage}%`;
      progressText.textContent = `${Number(payload.percentage).toFixed(2)}%`;
      sizeText.textContent = `已下载: ${formatFileSize(payload.downloaded)} / ${formatFileSize(payload.total)}`;
    }else if(step===2){
      hasUpdate.value=false;
      currentVersion.value=latestVersion.value
      console.log("正在启动 ...");
      error.value = "";
      Swal.close();
      await relaunch()
      // const result = await invoke("tg_start");
      // console.log("启动命令已发送",result);
    }else {
      Swal.close();
      await showError('更新失败','更新过程中发生错误');
    }
  });
};
const get_version_info = () => {
  invoke("get_version_info").then(async res=>{
    console.log("get_version_info",res);
    core_last_version.value = res.coreLastVersion;
    coreLastUrl.value = res.coreLastUrl;
    currentVersion.value = res.currentVersion;
    latestVersion.value = res.latestVersion;
    versionDetail.value = res.detail;
    if (res.detail && res.detail.url && res.detail.url[os_detail.value]){
      latestUrl.value=res.detail.url[os_detail.value];
    }
    core_has_update.value=core_current_version.value !== core_last_version.value && coreLastUrl.value;
    hasUpdate.value = currentVersion.value !== latestVersion.value && latestUrl.value && !core_has_update.value && latestVersion.value != '2.2.0';
    await invoke("tg_title",{version:currentVersion.value})
  });
}

onMounted(async () => {

  console.log("组件已挂载");
  // await setupListeners();
  // showLoading('启动中......',"系统正在启动")
  console.log("开始发送初始化命令")
  // const result = await invoke("tg_init");
  // os_detail.value=result.os_detail
  // core_current_version.value=result.core_current_version

  // console.log("初始化命令已发送",result);



  setTimeout(async ()=>{
    location.replace("http://localhost:3000");

    // if (result){
      await invoke("tg_start")
    // }
  },5000)

});

onUnmounted(() => {
  window.removeEventListener('resize', () => {});
  if (retryTimeoutId.value) {
    clearTimeout(retryTimeoutId.value);
  }
  if (loadTimeoutId.value) {
    clearTimeout(loadTimeoutId.value);
  }
});

const handleUpdate = async () => {
  if (!hasUpdate.value || !latestUrl.value) return;

  try {
    const result = await Swal.fire({
      title: '确认更新',
      text: `当前版本: ${currentVersion.value}\n最新版本: ${latestVersion.value}`,
      icon: 'info',
      showCancelButton: true,
      confirmButtonText: '更新',
      cancelButtonText: '取消'
    });
    
    // 如果用户点击了取消按钮，直接返回
    if (!result.isConfirmed) {
      return;
    }
    Swal.close()
    await invoke("perform_update",{url:latestUrl.value});
  } catch (error) {
    console.error('更新失败:', error);
    await showError('更新失败','更新过程中发生错误');
  }
};
const handleUpdateCore = async () => {
  if (!core_has_update || !coreLastUrl.value) return;
  try {
    await openUrl(coreLastUrl.value)
  } catch (error) {
    console.error('更新失败:', error);
    await showError('更新失败','更新过程中发生错误');
  }
};


// 添加文件大小格式化函数
const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// 开始iframe加载超时检测
const startIframeLoadTimeout = () => {
  // 清除之前的计时器
  if (loadTimeoutId.value) {
    clearTimeout(loadTimeoutId.value);
  }
  
  // 设置新的超时计时器
  loadTimeoutId.value = setTimeout(() => {
    // 如果计时器触发时iframe还未加载完成，则认为加载失败
    if (!iframeLoaded.value) {
      logs.value.push("应用界面加载超时，准备重试...");
      handleIframeLoadFailure();
    }
  }, IFRAME_LOAD_TIMEOUT);
};

const handleIframeLoad = () => {
  // 清除加载超时计时器
  if (loadTimeoutId.value) {
    clearTimeout(loadTimeoutId.value);
    loadTimeoutId.value = null;
  }
  
  // 设置短暂延迟后再将iframe标记为已加载，让加载动画完整显示
  setTimeout(() => {
    iframeLoaded.value = true;
    iframeRetryCount.value = 0;
    isRetrying.value = false;
    
    // 如果有重试计时器，清除它
    if (retryTimeoutId.value) {
      clearTimeout(retryTimeoutId.value);
      retryTimeoutId.value = null;
    }
    
    // 添加到日志
    logs.value.push("应用界面加载成功");
  }, 1500); // 1.5秒的延迟确保动画完整显示
};

const handleIframeLoadFailure = () => {
  // iframe加载失败，准备重试
  iframeLoaded.value = false;
  isRetrying.value = true;
  
  if (iframeRetryCount.value < MAX_RETRY_COUNT) {
    iframeRetryCount.value++;
    logs.value.push(`界面加载失败，正在进行第 ${iframeRetryCount.value} 次重试...`);
    
    // 设置重试计时器，每次延迟增加
    const retryDelay = 1000 + (iframeRetryCount.value * 500); // 递增延迟: 1.5s, 2s, 2.5s...
    
    if (retryTimeoutId.value) {
      clearTimeout(retryTimeoutId.value);
    }
    
    retryTimeoutId.value = setTimeout(() => {
    }, retryDelay);
  } else {
    isRetrying.value = false;
    logs.value.push("界面加载失败，已达到最大重试次数");
    error.value = "无法加载应用界面，请尝试重启程序";
  }
};

// 手动触发重试
const triggerRetry = () => {
  if (!iframeLoaded.value && !isRetrying.value) {
    // 重置重试计数
    iframeRetryCount.value = 0;
    logs.value.push("手动重试加载应用界面...");
    handleIframeLoadFailure();
  }
};

const toggleLogPopup = () => {
  showLogPopup.value = !showLogPopup.value;
  if (showLogPopup.value) {
    // 等待 DOM 更新后滚动到底部
    nextTick(() => {
      const logContainer = document.querySelector('.overflow-y-auto');
      if (logContainer) {
        logContainer.scrollTop = logContainer.scrollHeight;
      }
    });
  }
};

// 监听日志更新，自动滚动到底部
watch(logs, () => {
  if (showLogPopup.value) {
    nextTick(() => {
      const logContainer = document.querySelector('.overflow-y-auto');
      if (logContainer) {
        logContainer.scrollTop = logContainer.scrollHeight;
      }
    });
  }
});

// 接收来自 iframe 的消息
window.addEventListener("message", async (event)=> {
  const { type, payload } = event.data;
  if (type === "file_save") {
    const path =await save();
    if (path){
      event.source.postMessage({
        type:'file_open',
        dir: path,
        typeid: payload.typeid,
      }, "*");
    }
  }else if (type === "file_open") {
    const path =await open({
      multiple: false,
      directory: true,
    });
    if (path){
      event.source.postMessage({
        type:'file_open',
        dir: path,
        typeid: payload.typeid,
      }, "*");
    }
  }else if (type === "browser_open") {
    await openUrl(payload.url);
  }
});


</script>

<template>
  <div class="flex flex-col h-screen bg-gray-100">
    <!-- 更新提示 -->
    <div v-if="hasUpdate" class="bg-blue-50 border-blue-500 border-l-4 p-3 flex justify-between items-center">
      <div class="flex items-center">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-blue-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span class="text-blue-700">发现新版本: {{ latestVersion }} (当前版本: {{ currentVersion }})</span>
      </div>
      <div class="flex space-x-2">
        <button 
          class="border border-blue-500 text-blue-500 hover:bg-blue-50 px-3 py-1 rounded text-sm font-medium transition-colors"
          @click="hasUpdate = false"
        >
          稍后更新
        </button>
        <button 
          @click="handleUpdate" 
          class="bg-blue-500 hover:bg-blue-600 text-white px-3 py-1 rounded text-sm font-medium transition-colors"
        >
          立即更新
        </button>
      </div>
    </div>
     <!-- 内核更新提示 -->
    <div v-if="core_has_update" class="bg-blue-50 border-blue-500 border-l-4 p-3 flex justify-between items-center">
      <div class="flex items-center">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-blue-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span class="text-blue-700">内核新版本: {{ core_last_version }} (当前版本: {{ core_current_version }})</span>
      </div>
      <div class="flex space-x-2">
        <button
          class="border border-blue-500 text-blue-500 hover:bg-blue-50 px-3 py-1 rounded text-sm font-medium transition-colors"
          @click="core_has_update = false">
          暂不下载
        </button>
        <button
          @click="handleUpdateCore"
          class="bg-blue-500 hover:bg-blue-600 text-white px-3 py-1 rounded text-sm font-medium transition-colors">
          前往下载
        </button>
      </div>
    </div>

    
    <!-- 错误提示 -->
    <div v-if="error" class="bg-red-100 border-l-4 border-red-500 text-red-700 p-4 mb-4">
      <p>{{ error }}</p>
    </div>
    
    <!-- iframe 区域 - 添加 flex-1 让它占据剩余空间 -->
    <div class="flex-1 bg-white shadow-md overflow-auto relative">
      <!-- iframe加载中或重试中的UI -->
      <div v-if="status && !iframeLoaded" class="absolute inset-0 flex items-center justify-center bg-gradient-to-b from-blue-50 to-white bg-gradient-animate z-10">
        <div class="text-center w-[600px] p-12 bg-white/80 backdrop-blur-lg rounded-3xl shadow-[0_20px_60px_-10px_rgba(59,130,246,0.3)] border-2 border-blue-100/70 transition-all duration-500 animate-card-appear" style="max-height: 650px;">
          <!-- 添加Logo -->
          <div class="mb-4 animate-fade-in" style="animation-delay: 0.1s">
            <img :src="logo" alt="Logo" class="h-20 mx-auto drop-shadow-lg filter contrast-125 animate-logo-glow" />
          </div>
          
          <div class="mb-10 relative">
            <!-- 大型光晕效果 -->
            <div class="absolute inset-0 rounded-full bg-blue-200/40 blur-2xl transform scale-150 animate-pulse-slow"></div>
            
            <!-- 外圆 -->
            <div class="w-40 h-40 mx-auto rounded-full bg-blue-50/90 flex items-center justify-center p-2 shadow-xl shadow-blue-200/50 relative border border-blue-100/50">
              <!-- 内圆及动画 -->
              <svg class="animate-spin h-24 w-24 text-blue-500 mx-auto" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3"></circle>
                <path class="opacity-90" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>
            
            <!-- 装饰元素 - 更多、更大的装饰元素 -->
            <div class="absolute top-0 right-0 -mr-6 -mt-6 w-12 h-12 rounded-full bg-gradient-to-br from-blue-400 to-blue-500 opacity-70 animate-float"></div>
            <div class="absolute bottom-0 left-0 -ml-5 -mb-5 w-10 h-10 rounded-full bg-gradient-to-tr from-blue-300 to-indigo-400 opacity-60 animate-float" style="animation-delay: 0.5s"></div>
            <div class="absolute top-1/2 left-0 -ml-8 mt-3 w-8 h-8 rounded-full bg-gradient-to-r from-blue-500 to-indigo-500 opacity-50 animate-float" style="animation-delay: 0.8s"></div>
            <div class="absolute bottom-1/3 right-0 -mr-7 mb-10 w-9 h-9 rounded-full bg-gradient-to-l from-indigo-400 to-blue-400 opacity-60 animate-float" style="animation-delay: 1.2s"></div>
            <!-- 额外装饰元素 -->
            <div class="absolute top-1/4 right-1/4 w-6 h-6 rounded-full bg-gradient-to-tl from-sky-400 to-blue-300 opacity-50 animate-float" style="animation-delay: 1.5s"></div>
            <div class="absolute bottom-1/4 left-1/4 w-7 h-7 rounded-full bg-gradient-to-bl from-indigo-300 to-blue-400 opacity-40 animate-float" style="animation-delay: 0.9s"></div>
          </div>
          
          <!-- 状态显示 - 更大的标题 -->
          <h3 class="text-3xl font-bold text-gray-800 mb-6 animate-fade-in tracking-wide" style="animation-delay: 0.3s">
            {{ isRetrying ? '正在重试连接应用...' : '准备加载应用' }}
          </h3>
          
          <!-- 优雅的分隔线 -->
          <div class="w-24 h-1 mx-auto mb-6 rounded-full bg-gradient-to-r from-blue-300 to-indigo-400 opacity-70 animate-fade-in" style="animation-delay: 0.35s"></div>
          
          <!-- 重试信息 - 更丰富的视觉效果 -->
          <div v-if="isRetrying" class="mb-8 animate-fade-in mx-auto max-w-md" style="animation-delay: 0.4s">
            <div class="w-full bg-gray-100 rounded-full h-3 mb-4 overflow-hidden shadow-inner">
              <div class="bg-gradient-to-r from-blue-400 via-indigo-500 to-blue-600 h-3 rounded-full transition-all duration-500 ease-out animate-pulse-slow" 
                   :style="{ width: `${(iframeRetryCount / MAX_RETRY_COUNT) * 100}%` }"></div>
            </div>
            <p class="text-base text-gray-600">重试进度：{{ iframeRetryCount }}/{{ MAX_RETRY_COUNT }}</p>
          </div>
          <p v-else class="text-lg text-gray-600 mb-8 max-w-lg mx-auto animate-fade-in leading-relaxed" style="animation-delay: 0.4s">
            请稍候，精彩内容马上就绪...
          </p>
          
          <!-- 错误信息 - 更高级的卡片样式 -->
          <p v-if="error" class="mb-8 p-6 bg-red-50 rounded-xl text-base text-red-500 border-l-4 border-red-400 shadow-md max-w-lg mx-auto animate-fade-in">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 inline mr-2 mb-1 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            {{ error }}
          </p>
          
          <!-- 重试/退出按钮 - 更大、更华丽的按钮 -->
          <div v-if="iframeRetryCount >= MAX_RETRY_COUNT" class="flex justify-center space-x-6 mt-10 animate-fade-in" style="animation-delay: 0.5s">
            <button 
              @click="triggerRetry" 
              class="relative overflow-hidden bg-gradient-to-r from-blue-500 via-blue-600 to-indigo-600 hover:from-blue-600 hover:via-blue-700 hover:to-indigo-700 text-white px-8 py-4 rounded-xl text-base font-medium transition-all duration-300 shadow-lg hover:shadow-xl transform hover:-translate-y-1 flex items-center group"
            >
              <span class="absolute inset-0 bg-white/10 transform scale-x-0 group-hover:scale-x-100 transition-transform origin-left duration-300"></span>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 mr-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              重新加载
            </button>
            <button 
              @click="do_exit" 
              class="relative overflow-hidden bg-gradient-to-r from-red-500 via-red-600 to-red-700 hover:from-red-600 hover:via-red-700 hover:to-red-800 text-white px-8 py-4 rounded-xl text-base font-medium transition-all duration-300 shadow-lg hover:shadow-xl transform hover:-translate-y-1 flex items-center group"
            >
              <span class="absolute inset-0 bg-white/10 transform scale-x-0 group-hover:scale-x-100 transition-transform origin-left duration-300"></span>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 mr-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
              退出程序
            </button>
          </div>
          
          <!-- 加载指示器动画 -->
          <div v-if="!isRetrying && !error" class="mt-8 max-w-lg mx-auto animate-fade-in" style="animation-delay: 0.5s">
            <div class="flex justify-center space-x-2">
              <div class="loading-dot" style="animation-delay: 0s"></div>
              <div class="loading-dot" style="animation-delay: 0.2s"></div>
              <div class="loading-dot" style="animation-delay: 0.4s"></div>
              <div class="loading-dot" style="animation-delay: 0.6s"></div>
              <div class="loading-dot" style="animation-delay: 0.8s"></div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- iframe -->
<!--      <iframe-->
<!--        v-if="status"-->
<!--        :src="frame_url"-->
<!--        class="w-full h-full border-0"-->
<!--        title="Control Panel"-->
<!--        style="width: 100%;"-->
<!--        @load="handleIframeLoad"-->
<!--      ></iframe>-->
      
      <!-- 初始化等待 - 使用相同的高级设计 -->
      <div v-else class="w-full h-full flex items-center justify-center bg-gradient-to-b from-blue-50 to-white bg-gradient-animate">
        <div class="text-center w-[600px] p-12 bg-white/80 backdrop-blur-lg rounded-3xl shadow-[0_20px_60px_-10px_rgba(59,130,246,0.3)] border-2 border-blue-100/70 transition-all duration-500 animate-card-appear" style="max-height: 650px;">
          <!-- 添加Logo -->
          <div class="mb-4 animate-fade-in" style="animation-delay: 0.1s">
            <img :src="logo" alt="Logo" class="h-20 mx-auto drop-shadow-lg filter contrast-125 animate-logo-glow" />
          </div>
          
          <div class="mb-10 relative">
            <!-- 大型光晕效果 -->
            <div class="absolute inset-0 rounded-full bg-blue-200/40 blur-2xl transform scale-150 animate-pulse-slow"></div>
            
            <!-- 外圆 -->
            <div class="w-40 h-40 mx-auto rounded-full bg-blue-50/90 flex items-center justify-center p-2 shadow-xl shadow-blue-200/50 relative border border-blue-100/50">
              <!-- 内圆及动画 -->
              <svg class="animate-spin h-24 w-24 text-blue-500 mx-auto" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3"></circle>
                <path class="opacity-90" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>
            
            <!-- 装饰元素 - 更多、更大的装饰元素 -->
            <div class="absolute top-0 right-0 -mr-6 -mt-6 w-12 h-12 rounded-full bg-gradient-to-br from-blue-400 to-blue-500 opacity-70 animate-float"></div>
            <div class="absolute bottom-0 left-0 -ml-5 -mb-5 w-10 h-10 rounded-full bg-gradient-to-tr from-blue-300 to-indigo-400 opacity-60 animate-float" style="animation-delay: 0.5s"></div>
            <div class="absolute top-1/2 left-0 -ml-8 mt-3 w-8 h-8 rounded-full bg-gradient-to-r from-blue-500 to-indigo-500 opacity-50 animate-float" style="animation-delay: 0.8s"></div>
            <div class="absolute bottom-1/3 right-0 -mr-7 mb-10 w-9 h-9 rounded-full bg-gradient-to-l from-indigo-400 to-blue-400 opacity-60 animate-float" style="animation-delay: 1.2s"></div>
            <!-- 额外装饰元素 -->
            <div class="absolute top-1/4 right-1/4 w-6 h-6 rounded-full bg-gradient-to-tl from-sky-400 to-blue-300 opacity-50 animate-float" style="animation-delay: 1.5s"></div>
            <div class="absolute bottom-1/4 left-1/4 w-7 h-7 rounded-full bg-gradient-to-bl from-indigo-300 to-blue-400 opacity-40 animate-float" style="animation-delay: 0.9s"></div>
          </div>
          <h3 class="text-3xl font-bold text-gray-800 mb-6 animate-fade-in tracking-wide" style="animation-delay: 0.3s">应用正在启动</h3>
          
          <!-- 优雅的分隔线 -->
          <div class="w-24 h-1 mx-auto mb-6 rounded-full bg-gradient-to-r from-blue-300 to-indigo-400 opacity-70 animate-fade-in" style="animation-delay: 0.35s"></div>
          
          <p class="text-lg text-gray-600 max-w-lg mx-auto mb-10 leading-relaxed animate-fade-in" style="animation-delay: 0.4s">耐心等待片刻，我们正在为您准备精彩内容...</p>
          
          <!-- 高级加载进度条 -->
          <div class="mt-8 max-w-lg mx-auto animate-fade-in" style="animation-delay: 0.5s">
            <div class="w-full h-3 bg-gray-100 rounded-full overflow-hidden shadow-inner">
              <div class="h-full loading-progress-bar"></div>
            </div>
            <!-- 加载指示器动画 -->
            <div class="flex justify-center space-x-2 mt-8">
              <div class="loading-dot" style="animation-delay: 0s"></div>
              <div class="loading-dot" style="animation-delay: 0.2s"></div>
              <div class="loading-dot" style="animation-delay: 0.4s"></div>
              <div class="loading-dot" style="animation-delay: 0.6s"></div>
              <div class="loading-dot" style="animation-delay: 0.8s"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 分隔线 -->
    <div class="border-b"></div>
    
    <!-- 底部状态栏 -->
    <div class="bg-white py-1 px-4 flex justify-between items-center h-8">
      <!-- 左侧日志显示 -->
      <div class="flex-1 overflow-hidden relative">
        <!-- 日志弹窗 -->
        <div v-if="showLogPopup" 
          class="fixed bottom-10 left-1 w-1/2 max-h-64 bg-white rounded-lg shadow-xl border border-gray-200 z-50">
          <div class="flex flex-col h-full">
            <div class="flex justify-between items-center p-3 border-b bg-gray-50">
              <span class="text-sm font-medium text-gray-700">日志记录</span>
              <button @click="toggleLogPopup" class="text-gray-500 hover:text-gray-700">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div class="overflow-y-auto p-3" style="height: 180px;" ref="logContainer">
              <div v-for="(log, index) in logs" :key="index" class="text-sm text-gray-800 py-1 border-b border-gray-100 last:border-0">
                {{ log }}
              </div>
            </div>
          </div>
        </div>
        <!-- 日志显示区域 -->
        <div class="log-scroll cursor-pointer" :key="logs.length" @click="toggleLogPopup">
          <p class="text-sm font-medium text-gray-800">
            {{ logs.length > 0 ? logs[logs.length - 1] : '等待应用日志...' }}
          </p>
        </div>
      </div>
      <!-- 右侧退出按钮 -->
      <div class="ml-4">
        <button @click="do_exit" class="bg-blue-500 hover:bg-blue-600 text-white px-2 py-0.5 rounded text-sm font-medium transition-colors">
          退出程序
        </button>
      </div>
    </div>
  </div>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
}

body {
  margin: 0;
  min-width: 320px;
  min-height: 100vh;
}

button {
  cursor: pointer;
  transition: all 0.3s;
}

button:disabled {
  cursor: not-allowed;
}

input {
  font-family: inherit;
}

.log-scroll {
  animation: scrollText 0.3s ease-out;
  transform-origin: top;
  transition: background-color 0.2s;
  padding: 2px 4px;
  border-radius: 4px;
}

.log-scroll:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.log-scroll p {
  text-shadow: 0 0 1px rgba(0, 0, 0, 0.1);
  letter-spacing: 0.3px;
}

@keyframes scrollText {
  0% {
    transform: translateY(100%);
    opacity: 0;
  }
  100% {
    transform: translateY(0);
    opacity: 1;
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1.2s linear infinite;
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.animate-pulse-slow {
  animation: pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 0.6;
  }
  50% {
    opacity: 0.3;
  }
}

/* 卡片出现动画 */
.animate-card-appear {
  animation: cardAppear 1s cubic-bezier(0.19, 1, 0.22, 1) forwards;
}

@keyframes cardAppear {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.95);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* 淡入动画 */
.animate-fade-in {
  animation: fadeIn 0.8s ease-in-out forwards;
  opacity: 0;
}

@keyframes fadeIn {
  0% {
    opacity: 0;
    transform: translateY(15px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 悬浮动画 */
.animate-float {
  animation: float 4s ease-in-out infinite;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-12px);
  }
}

/* 进度条加载动画 */
.loading-progress-bar {
  width: 30%;
  animation: loading 3s ease-in-out infinite;
  background-image: linear-gradient(to right, #3b82f6, #6366f1, #4f46e5, #6366f1, #3b82f6);
  background-size: 300% 100%;
}

@keyframes loading {
  0% {
    width: 0%;
    background-position: 100% 50%;
  }
  50% {
    width: 75%;
    background-position: 0% 50%;
  }
  100% {
    width: 98%;
    background-position: 100% 50%;
  }
}

/* 渐变动画 */
.bg-gradient-animate {
  background-size: 400% 400%;
  animation: gradient 15s ease infinite;
  background-image: linear-gradient(135deg, 
    rgba(239, 246, 255, 0.95), 
    rgba(219, 234, 254, 0.9), 
    rgba(191, 219, 254, 0.85), 
    rgba(147, 197, 253, 0.8),
    rgba(96, 165, 250, 0.75),
    rgba(59, 130, 246, 0.7),
    rgba(37, 99, 235, 0.65),
    rgba(59, 130, 246, 0.7),
    rgba(96, 165, 250, 0.75),
    rgba(147, 197, 253, 0.8),
    rgba(191, 219, 254, 0.85),
    rgba(219, 234, 254, 0.9),
    rgba(239, 246, 255, 0.95)
  );
}

@keyframes gradient {
  0% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
  100% {
    background-position: 0% 50%;
  }
}

/* 加载点动画 */
.loading-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: #3b82f6;
  animation: loadingDot 1.4s ease-in-out infinite;
}

@keyframes loadingDot {
  0%, 100% {
    transform: translateY(0);
    opacity: 0.3;
  }
  50% {
    transform: translateY(-10px);
    opacity: 1;
  }
}

@media (prefers-color-scheme: dark) {
  .bg-gray-100 {
    background-color: #1a1a1a;
  }
  
  .bg-white {
    background-color: #2d2d2d;
  }
}

/* Logo动画效果 */
.animate-logo-glow {
  animation: logoGlow 2.5s ease-in-out infinite;
  transform-origin: center;
}

@keyframes logoGlow {
  0%, 100% {
    filter: drop-shadow(0 0 8px rgba(59, 130, 246, 0.3));
    transform: scale(1);
  }
  50% {
    filter: drop-shadow(0 0 15px rgba(59, 130, 246, 0.5));
    transform: scale(1.05);
  }
}
</style>
