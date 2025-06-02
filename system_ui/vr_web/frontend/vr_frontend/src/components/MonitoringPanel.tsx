import React, { useState, useEffect } from 'react';
import { monitoringApi } from '@/lib/api';
import { 
  Card, 
  CardContent, 
  CardDescription, 
  CardHeader, 
  CardTitle 
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ReloadIcon } from "@radix-ui/react-icons";

export const MonitoringPanel: React.FC = () => {
  const [monitoringStatus, setMonitoringStatus] = useState<{
    status: string;
    message: string;
  } | null>(null);
  
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchMonitoringStatus();
  }, []);

  const fetchMonitoringStatus = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await monitoringApi.getMonitoringStatus();
      setMonitoringStatus(response);
    } catch (err) {
      console.error('Failed to fetch monitoring status:', err);
      setError('Failed to load monitoring status. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      {error && (
        <Alert variant="destructive">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      
      <Card>
        <CardHeader>
          <CardTitle>System Monitoring</CardTitle>
          <CardDescription>Real-time system performance and status</CardDescription>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="flex justify-center py-4">
              <ReloadIcon className="h-6 w-6 animate-spin" />
            </div>
          ) : monitoringStatus ? (
            <div className="space-y-4">
              <div className="text-center p-8 border rounded-md">
                <p className="text-lg">{monitoringStatus.message}</p>
                <p className="text-sm text-muted-foreground mt-2">
                  This feature will provide real-time monitoring of system performance, 
                  resource usage, and component status in future updates.
                </p>
              </div>
              
              <Button 
                variant="outline" 
                size="sm" 
                onClick={fetchMonitoringStatus}
                className="w-full"
              >
                Refresh Status
              </Button>
            </div>
          ) : (
            <div className="text-center py-4">
              No monitoring data available.
            </div>
          )}
        </CardContent>
      </Card>
      
      {/* Placeholder for future monitoring components */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>CPU & Memory</CardTitle>
            <CardDescription>Resource utilization</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-center p-4 border rounded-md">
              <p className="text-sm text-muted-foreground">
                CPU and memory monitoring will be available in future updates.
              </p>
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader>
            <CardTitle>Temperature & Power</CardTitle>
            <CardDescription>Thermal and power monitoring</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-center p-4 border rounded-md">
              <p className="text-sm text-muted-foreground">
                Temperature and power monitoring will be available in future updates.
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};
