# Vulkan Validation Warnings - Known Issues

## VUID-vkQueueSubmit-pSignalSemaphores-00067

### Problem Description
When running StratoSDK applications, you might see Vulkan validation warnings similar to:

```
VALIDATION [VUID-vkQueueSubmit-pSignalSemaphores-00067] vkQueueSubmit(): 
pSubmits[0].pSignalSemaphores[0] (VkSemaphore 0x...) is being signaled by VkQueue 0x..., 
but it may still be in use by VkSwapchainKHR 0x...
```

### Cause
This is a **known issue in WGPU** (the rendering backend used by StratoSDK) related to unsafe reuse of semaphores in the Vulkan swapchain. The problem occurs because:

1. WGPU reuses the same semaphore for different frames before the previous presentation operation is completed
2. Synchronization is based on frames in flight rather than swapchain images
3. There are no guarantees that previous presentation operations are completed

### Impact
- **Functionality**: The application continues to work correctly
- **Performance**: No significant impact on performance
- **Stability**: No crashes or undefined behavior observed
- **Validation**: Vulkan validation warnings (not fatal errors)

### Resolution Status
This is an issue that must be resolved **upstream in WGPU**:
- Tracked issues: [#5559](https://github.com/gfx-rs/wgpu/issues/5559), [#7957](https://github.com/gfx-rs/wgpu/issues/7957)
- Requires changes to WGPU's HAL (Hardware Abstraction Layer)
- Cannot be resolved at the application level

### Temporary Solutions
To reduce warnings during development:

1. **Disable Vulkan validation** (development only):
   ```bash
   # Windows
   set VK_INSTANCE_LAYERS=
   
   # Linux/macOS
   export VK_INSTANCE_LAYERS=
   ```

2. **Use DX12 backend on Windows** (if available):
   ```rust
   let backends = wgpu::Backends::DX12;
   ```

### Notes for Developers
- These warnings do not indicate errors in the application code
- The problem is limited to WGPU's Vulkan backend
- Other backends (DX12, Metal) are not affected by this issue
- The definitive resolution will come with future versions of WGPU

---
*Last updated: January 2025*
*WGPU Version: 0.20+*