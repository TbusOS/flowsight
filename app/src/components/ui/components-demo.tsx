import React, { useState } from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Input,
  Badge,
} from "../../components/ui";
import "./components-demo.css";

export function ComponentsDemo() {
  const [open, setOpen] = useState(false);
  const [inputValue, setInputValue] = useState("");

  return (
    <div className="demo-container">
      <div className="demo-header">
        <h1>FlowSight UI Components</h1>
        <p>shadcn/ui 风格组件演示</p>
      </div>

      <div className="demo-section">
        <h2>Buttons</h2>
        <div className="demo-row">
          <Button variant="default">Default</Button>
          <Button variant="secondary">Secondary</Button>
          <Button variant="outline">Outline</Button>
          <Button variant="ghost">Ghost</Button>
          <Button variant="destructive">Destructive</Button>
        </div>
        <div className="demo-row">
          <Button size="sm">Small</Button>
          <Button size="default">Default</Button>
          <Button size="lg">Large</Button>
        </div>
      </div>

      <div className="demo-section">
        <h2>Input</h2>
        <div className="demo-row">
          <Input
            placeholder="Search..."
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
          />
        </div>
      </div>

      <div className="demo-section">
        <h2>Badges</h2>
        <div className="demo-row">
          <Badge>Default</Badge>
          <Badge variant="secondary">Secondary</Badge>
          <Badge variant="success">Success</Badge>
          <Badge variant="warning">Warning</Badge>
          <Badge variant="destructive">Error</Badge>
          <Badge variant="outline">Outline</Badge>
        </div>
      </div>

      <div className="demo-section">
        <h2>Tabs</h2>
        <Tabs defaultValue="tab1" className="demo-tabs">
          <TabsList>
            <TabsTrigger value="tab1">Overview</TabsTrigger>
            <TabsTrigger value="tab2">Details</TabsTrigger>
            <TabsTrigger value="tab3">Settings</TabsTrigger>
          </TabsList>
          <TabsContent value="tab1">
            <Card>
              <CardHeader>
                <CardTitle>Overview</CardTitle>
                <CardDescription>
                  This is the overview content for the first tab.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <p>Dashboard metrics and statistics go here.</p>
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="tab2">
            <Card>
              <CardHeader>
                <CardTitle>Details</CardTitle>
                <CardDescription>
                  Detailed information about the selected item.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <p>Detailed view content.</p>
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="tab3">
            <Card>
              <CardHeader>
                <CardTitle>Settings</CardTitle>
                <CardDescription>
                  Configure your preferences.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <p>Settings options go here.</p>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>

      <div className="demo-section">
        <h2>Cards</h2>
        <div className="demo-cards">
          <Card>
            <CardHeader>
              <CardTitle>Card Title</CardTitle>
              <CardDescription>Card description text goes here.</CardDescription>
            </CardHeader>
            <CardContent>
              <p>Card content area with various elements.</p>
            </CardContent>
            <CardFooter>
              <Button variant="outline" size="sm">
                Cancel
              </Button>
              <Button size="sm">Confirm</Button>
            </CardFooter>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Statistics</CardTitle>
              <CardDescription>Performance metrics</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="stat-grid">
                <div className="stat-item">
                  <span className="stat-value">128</span>
                  <span className="stat-label">Functions</span>
                </div>
                <div className="stat-item">
                  <span className="stat-value">64</span>
                  <span className="stat-label">Callsites</span>
                </div>
                <div className="stat-item">
                  <span className="stat-value">12</span>
                  <span className="stat-label">Scenarios</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <div className="demo-section">
        <h2>Dialog</h2>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button>Open Dialog</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Confirm Action</DialogTitle>
              <DialogDescription>
                Are you sure you want to proceed with this action? This cannot
                be undone.
              </DialogDescription>
            </DialogHeader>
            <div className="py-4">
              <Input placeholder="Enter confirmation..." />
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setOpen(false)}>
                Cancel
              </Button>
              <Button onClick={() => setOpen(false)}>Confirm</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="demo-section">
        <h2>Interactive Example</h2>
        <Card>
          <CardHeader>
            <CardTitle>Function Analysis</CardTitle>
            <CardDescription>
              <Badge variant="secondary">async void(*)(int)</Badge>
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="analysis-form">
              <div className="form-group">
                <label>Function Name</label>
                <Input defaultValue="handle_interrupt" />
              </div>
              <div className="form-group">
                <label>Parameters</label>
                <Input defaultValue="irq_handler_t handler" />
              </div>
              <div className="form-group">
                <label>Return Type</label>
                <Input defaultValue="int" />
              </div>
            </div>
          </CardContent>
          <CardFooter>
            <Button variant="ghost">Cancel</Button>
            <Button>Save Changes</Button>
          </CardFooter>
        </Card>
      </div>
    </div>
  );
}

export default ComponentsDemo;
