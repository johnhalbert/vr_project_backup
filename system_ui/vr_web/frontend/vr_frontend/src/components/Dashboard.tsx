import React from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ConfigPanel } from './ConfigPanel';
import { HardwarePanel } from './HardwarePanel';
import { SystemPanel } from './SystemPanel';
import { MonitoringPanel } from './MonitoringPanel';

export const Dashboard: React.FC = () => {
  return (
    <div className="container mx-auto p-4">
      <h1 className="text-3xl font-bold mb-6">VR Headset Control Panel</h1>
      
      <Tabs defaultValue="config" className="w-full">
        <TabsList className="grid grid-cols-4 mb-4">
          <TabsTrigger value="config">Configuration</TabsTrigger>
          <TabsTrigger value="hardware">Hardware</TabsTrigger>
          <TabsTrigger value="system">System</TabsTrigger>
          <TabsTrigger value="monitoring">Monitoring</TabsTrigger>
        </TabsList>
        
        <TabsContent value="config">
          <Card>
            <CardHeader>
              <CardTitle>Configuration</CardTitle>
              <CardDescription>
                Manage VR headset configuration settings
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ConfigPanel />
            </CardContent>
          </Card>
        </TabsContent>
        
        <TabsContent value="hardware">
          <Card>
            <CardHeader>
              <CardTitle>Hardware</CardTitle>
              <CardDescription>
                Manage and monitor hardware devices
              </CardDescription>
            </CardHeader>
            <CardContent>
              <HardwarePanel />
            </CardContent>
          </Card>
        </TabsContent>
        
        <TabsContent value="system">
          <Card>
            <CardHeader>
              <CardTitle>System</CardTitle>
              <CardDescription>
                System status and management
              </CardDescription>
            </CardHeader>
            <CardContent>
              <SystemPanel />
            </CardContent>
          </Card>
        </TabsContent>
        
        <TabsContent value="monitoring">
          <Card>
            <CardHeader>
              <CardTitle>Monitoring</CardTitle>
              <CardDescription>
                Real-time system monitoring
              </CardDescription>
            </CardHeader>
            <CardContent>
              <MonitoringPanel />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};
