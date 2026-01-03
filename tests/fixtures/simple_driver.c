/*
 * Simple USB Driver - Test Fixture for FlowSight
 * 
 * This is a minimal USB driver that demonstrates various patterns
 * that FlowSight should be able to analyze.
 */

#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/usb.h>
#include <linux/slab.h>

#define VENDOR_ID  0x1234
#define PRODUCT_ID 0x5678

/* Device-specific structure */
struct my_usb_device {
    struct usb_device *udev;
    struct usb_interface *interface;
    struct work_struct work;
    struct timer_list timer;
    int status;
    char *buffer;
    size_t buffer_size;
};

/* Forward declarations */
static void my_work_handler(struct work_struct *work);
static void my_timer_handler(struct timer_list *t);

/* Work queue handler - runs in process context */
static void my_work_handler(struct work_struct *work)
{
    struct my_usb_device *dev = container_of(work, struct my_usb_device, work);
    
    printk(KERN_INFO "Work handler called, status=%d\n", dev->status);
    
    /* Do some processing */
    if (dev->buffer) {
        memset(dev->buffer, 0, dev->buffer_size);
    }
    
    /* Reschedule timer */
    mod_timer(&dev->timer, jiffies + HZ);
}

/* Timer handler - runs in soft IRQ context */
static void my_timer_handler(struct timer_list *t)
{
    struct my_usb_device *dev = from_timer(dev, t, timer);
    
    printk(KERN_INFO "Timer fired, scheduling work\n");
    
    /* Schedule work from timer context */
    schedule_work(&dev->work);
}

/* Probe function - called when device is connected */
static int my_probe(struct usb_interface *interface,
                    const struct usb_device_id *id)
{
    struct my_usb_device *dev;
    struct usb_device *udev = interface_to_usbdev(interface);
    int ret;

    printk(KERN_INFO "USB device connected: %04x:%04x\n",
           id->idVendor, id->idProduct);

    /* Allocate device structure */
    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    /* Allocate buffer */
    dev->buffer_size = 4096;
    dev->buffer = kmalloc(dev->buffer_size, GFP_KERNEL);
    if (!dev->buffer) {
        ret = -ENOMEM;
        goto error_buffer;
    }

    /* Initialize device structure */
    dev->udev = usb_get_dev(udev);
    dev->interface = interface;
    dev->status = 0;

    /* Initialize work queue */
    INIT_WORK(&dev->work, my_work_handler);

    /* Initialize timer */
    timer_setup(&dev->timer, my_timer_handler, 0);

    /* Save device data */
    usb_set_intfdata(interface, dev);

    /* Start timer */
    mod_timer(&dev->timer, jiffies + HZ * 5);

    printk(KERN_INFO "Device initialized successfully\n");
    return 0;

error_buffer:
    kfree(dev);
    return ret;
}

/* Disconnect function - called when device is removed */
static void my_disconnect(struct usb_interface *interface)
{
    struct my_usb_device *dev = usb_get_intfdata(interface);

    printk(KERN_INFO "USB device disconnected\n");

    /* Stop timer */
    del_timer_sync(&dev->timer);

    /* Cancel pending work */
    cancel_work_sync(&dev->work);

    /* Release USB device reference */
    usb_put_dev(dev->udev);

    /* Free resources */
    kfree(dev->buffer);
    kfree(dev);
}

/* USB device ID table */
static const struct usb_device_id my_id_table[] = {
    { USB_DEVICE(VENDOR_ID, PRODUCT_ID) },
    { }
};
MODULE_DEVICE_TABLE(usb, my_id_table);

/* USB driver structure */
static struct usb_driver my_driver = {
    .name = "my_usb_driver",
    .id_table = my_id_table,
    .probe = my_probe,
    .disconnect = my_disconnect,
};

/* Module init */
static int __init my_init(void)
{
    printk(KERN_INFO "Loading my USB driver\n");
    return usb_register(&my_driver);
}

/* Module exit */
static void __exit my_exit(void)
{
    printk(KERN_INFO "Unloading my USB driver\n");
    usb_deregister(&my_driver);
}

module_init(my_init);
module_exit(my_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("FlowSight Test");
MODULE_DESCRIPTION("Simple USB driver for testing FlowSight");

