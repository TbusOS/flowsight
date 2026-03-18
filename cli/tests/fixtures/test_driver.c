/*
 * Test platform driver for FlowSight integration tests.
 *
 * A minimal but realistic Linux kernel platform driver that exercises
 * the main analysis features: structs, probe/remove callbacks,
 * IRQ handlers, workqueues, spinlocks, clk, ioremap, etc.
 */

#include <linux/module.h>
#include <linux/platform_device.h>
#include <linux/clk.h>
#include <linux/io.h>
#include <linux/interrupt.h>
#include <linux/workqueue.h>

struct my_device {
	void __iomem *base;
	struct clk *clk;
	int irq;
	struct work_struct work;
	spinlock_t lock;
};

static void my_work_handler(struct work_struct *work)
{
	struct my_device *dev = container_of(work, struct my_device, work);
	unsigned long flags;

	spin_lock_irqsave(&dev->lock, flags);
	writel(0x1, dev->base + 0x10);
	spin_unlock_irqrestore(&dev->lock, flags);
}

static irqreturn_t my_irq_handler(int irq, void *data)
{
	struct my_device *dev = data;
	u32 status = readl(dev->base + 0x00);

	if (status & 0x1) {
		schedule_work(&dev->work);
		return IRQ_HANDLED;
	}
	return IRQ_NONE;
}

static int my_device_probe(struct platform_device *pdev)
{
	struct my_device *dev;
	struct resource *res;
	int ret;

	dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
	if (!dev)
		return -ENOMEM;

	res = platform_get_resource(pdev, IORESOURCE_MEM, 0);
	dev->base = devm_ioremap_resource(&pdev->dev, res);
	if (IS_ERR(dev->base))
		return PTR_ERR(dev->base);

	dev->clk = devm_clk_get(&pdev->dev, NULL);
	if (IS_ERR(dev->clk))
		return PTR_ERR(dev->clk);

	ret = clk_prepare_enable(dev->clk);
	if (ret)
		return ret;

	dev->irq = platform_get_irq(pdev, 0);
	if (dev->irq < 0) {
		ret = dev->irq;
		goto err_clk;
	}

	spin_lock_init(&dev->lock);
	INIT_WORK(&dev->work, my_work_handler);

	ret = devm_request_irq(&pdev->dev, dev->irq, my_irq_handler,
			       0, "my-device", dev);
	if (ret)
		goto err_clk;

	platform_set_drvdata(pdev, dev);
	return 0;

err_clk:
	clk_disable_unprepare(dev->clk);
	return ret;
}

static int my_device_remove(struct platform_device *pdev)
{
	struct my_device *dev = platform_get_drvdata(pdev);
	cancel_work_sync(&dev->work);
	clk_disable_unprepare(dev->clk);
	return 0;
}

static const struct of_device_id my_device_dt_ids[] = {
	{ .compatible = "vendor,my-device" },
	{ }
};
MODULE_DEVICE_TABLE(of, my_device_dt_ids);

static struct platform_driver my_device_driver = {
	.probe = my_device_probe,
	.remove = my_device_remove,
	.driver = {
		.name = "my-device",
		.of_match_table = my_device_dt_ids,
	},
};
module_platform_driver(my_device_driver);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Test");
MODULE_DESCRIPTION("Test platform driver");
