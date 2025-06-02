import React, { useState, useEffect } from 'react';
import { systemApi } from '@/lib/api';
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
import { Badge } from "@/components/ui/badge";
import { 
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";

interface SystemComponent {
  name: string;
  status: string;
  details?: string;
}

interface SystemUpdate {
  component: string;
  current_version: string;
  available_version: string;
  description: string;
}

export const SystemPanel: React.FC = () => {
  const [statusData, setStatusData] = useState<{
    version: string;
    uptime: number;
    components: SystemComponent[];
  } | null>(null);
  
  const [infoData, setInfoData] = useState<{
    version: string;
    board_type: string;
    memory_size: string;
    os_version: string;
    kernel_version: string;
    config_path: string;
    log_path: string;
  } | null>(null);
  
  const [availableUpdates, setAvailableUpdates] = useState<SystemUpdate[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [checkingUpdates, setCheckingUpdates] = useState<boolean>(false);

  useEffect(() => {
    fetchSystemData();
  }, []);

  const fetchSystemData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [statusResponse, infoResponse] = await Promise.all([
        systemApi.getSystemStatus(),
        systemApi.getSystemInfo()
      ]);
      setStatusData(statusResponse);
      setInfoData(infoResponse);
    } catch (err) {
      console.error('Failed to fetch system data:', err);
      setError('Failed to load system data. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleCheckUpdates = async () => {
    setCheckingUpdates(true);
    setError(null);
    try {
      const response = await systemApi.updateSystem(true);
      if (response.available_updates) {
        setAvailableUpdates(response.available_updates);
      } else {
        setAvailableUpdates([]);
      }
    } catch (err) {
      console.error('Failed to check for updates:', err);
      setError('Failed to check for updates. Please try again.');
    } finally {
      setCheckingUpdates(false);
    }
  };

  const handleInstallUpdates = async () => {
    if (!window.confirm('Are you sure you want to install updates? This may restart the system.')) {
      return;
    }
    
    setLoading(true);
    setError(null);
    try {
      const response = await systemApi.updateSystem(false);
      alert(response.message);
      // Refresh system data after update
      fetchSystemData();
    } catch (err) {
      console.error('Failed to install updates:', err);
      setError('Failed to install updates. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleRestartSystem = async () => {
    if (!window.confirm('Are you sure you want to restart the system?')) {
      return;
    }
    
    setLoading(true);
    setError(null);
    try {
      const response = await systemApi.restartSystem(false);
      alert(response.message);
    } catch (err) {
      console.error('Failed to restart system:', err);
      setError('Failed to restart system. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;
    
    return `${hours}h ${minutes}m ${remainingSeconds}s`;
  };

  const getStatusBadge = (status: string) => {
    if (status === 'OK' || status === 'Connected') {
      return <Badge variant="success">{status}</Badge>;
    } else if (status.includes('Not')) {
      return <Badge variant="outline">{status}</Badge>;
    } else if (status.includes('Modified')) {
      return <Badge variant="warning">{status}</Badge>;
    } else {
      return <Badge>{status}</Badge>;
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
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* System Status Card */}
        <Card>
          <CardHeader>
            <CardTitle>System Status</CardTitle>
            <CardDescription>Current system status and health</CardDescription>
          </CardHeader>
          <CardContent>
            {loading && !statusData ? (
              <div className="flex justify-center py-4">
                <ReloadIcon className="h-6 w-6 animate-spin" />
              </div>
            ) : statusData ? (
              <div className="space-y-4">
                <div className="flex justify-between">
                  <span className="font-medium">Version:</span>
                  <span>{statusData.version}</span>
                </div>
                <div className="flex justify-between">
                  <span className="font-medium">Uptime:</span>
                  <span>{formatUptime(statusData.uptime)}</span>
                </div>
                <div className="border-t pt-4">
                  <h4 className="font-medium mb-2">Components</h4>
                  <div className="space-y-2">
                    {statusData.components.map((component, index) => (
                      <div key={index} className="flex justify-between items-center">
                        <span>{component.name}</span>
                        <div className="flex items-center">
                          {getStatusBadge(component.status)}
                          {component.details && (
                            <span className="ml-2 text-sm text-muted-foreground">
                              {component.details}
                            </span>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={fetchSystemData}
                  className="w-full"
                >
                  Refresh Status
                </Button>
              </div>
            ) : (
              <div className="text-center py-4">
                No status data available.
              </div>
            )}
          </CardContent>
        </Card>
        
        {/* System Info Card */}
        <Card>
          <CardHeader>
            <CardTitle>System Information</CardTitle>
            <CardDescription>Hardware and software details</CardDescription>
          </CardHeader>
          <CardContent>
            {loading && !infoData ? (
              <div className="flex justify-center py-4">
                <ReloadIcon className="h-6 w-6 animate-spin" />
              </div>
            ) : infoData ? (
              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="font-medium">Version:</span>
                  <span>{infoData.version}</span>
                </div>
                <div className="flex justify-between">
                  <span className="font-medium">Board Type:</span>
                  <span>{infoData.board_type}</span>
                </div>
                <div className="flex justify-between">
                  <span className="font-medium">Memory Size:</span>
                  <span>{infoData.memory_size}</span>
                </div>
                <div className="flex justify-between">
                  <span className="font-medium">OS Version:</span>
                  <span>{infoData.os_version}</span>
                </div>
                <div className="flex justify-between">
                  <span className="font-medium">Kernel Version:</span>
                  <span>{infoData.kernel_version}</span>
                </div>
                <div className="flex justify-between">
                  <span className="font-medium">Config Path:</span>
                  <span className="text-sm truncate max-w-[200px]">{infoData.config_path}</span>
                </div>
                <div className="flex justify-between">
                  <span className="font-medium">Log Path:</span>
                  <span className="text-sm truncate max-w-[200px]">{infoData.log_path}</span>
                </div>
              </div>
            ) : (
              <div className="text-center py-4">
                No system information available.
              </div>
            )}
          </CardContent>
        </Card>
      </div>
      
      {/* System Actions */}
      <Card>
        <CardHeader>
          <CardTitle>System Actions</CardTitle>
          <CardDescription>Manage system updates and operations</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col space-y-4">
            <div className="flex space-x-4">
              <Dialog>
                <DialogTrigger asChild>
                  <Button 
                    variant="outline" 
                    onClick={handleCheckUpdates}
                    disabled={checkingUpdates || loading}
                  >
                    {checkingUpdates ? (
                      <>
                        <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                        Checking...
                      </>
                    ) : (
                      'Check for Updates'
                    )}
                  </Button>
                </DialogTrigger>
                <DialogContent className="sm:max-w-[500px]">
                  <DialogHeader>
                    <DialogTitle>Available Updates</DialogTitle>
                    <DialogDescription>
                      The following updates are available for your system.
                    </DialogDescription>
                  </DialogHeader>
                  
                  {availableUpdates.length === 0 ? (
                    <div className="text-center py-4">
                      Your system is up to date.
                    </div>
                  ) : (
                    <div className="space-y-4 mt-4">
                      {availableUpdates.map((update, index) => (
                        <div key={index} className="border rounded-md p-3">
                          <div className="flex justify-between items-center mb-2">
                            <h4 className="font-medium">{update.component}</h4>
                            <Badge variant="outline">
                              {update.current_version} → {update.available_version}
                            </Badge>
                          </div>
                          <p className="text-sm text-muted-foreground">{update.description}</p>
                        </div>
                      ))}
                      
                      <Button 
                        onClick={handleInstallUpdates}
                        disabled={loading}
                        className="w-full"
                      >
                        Install Updates
                      </Button>
                    </div>
                  )}
                </DialogContent>
              </Dialog>
              
              <Button 
                variant="destructive" 
                onClick={handleRestartSystem}
                disabled={loading}
              >
                Restart System
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
