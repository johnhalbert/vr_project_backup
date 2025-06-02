import React, { useState, useEffect } from 'react';
import { configApi } from '@/lib/api';
import { 
  Table, 
  TableBody, 
  TableCaption, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from "@/components/ui/table";
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ReloadIcon } from "@radix-ui/react-icons";

interface ConfigItem {
  category: string;
  key: string;
  value: string;
  value_type: string;
}

export const ConfigPanel: React.FC = () => {
  const [configItems, setConfigItems] = useState<ConfigItem[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [editMode, setEditMode] = useState<boolean>(false);
  const [editItem, setEditItem] = useState<ConfigItem | null>(null);
  const [newValue, setNewValue] = useState<string>('');

  const categories = [
    { value: '', label: 'All Categories' },
    { value: 'hardware', label: 'Hardware' },
    { value: 'display', label: 'Display' },
    { value: 'audio', label: 'Audio' },
    { value: 'tracking', label: 'Tracking' },
    { value: 'network', label: 'Network' },
    { value: 'power', label: 'Power' },
    { value: 'steamvr', label: 'SteamVR' },
    { value: 'security', label: 'Security' },
    { value: 'system', label: 'System' },
  ];

  useEffect(() => {
    fetchConfigItems();
  }, [selectedCategory]);

  const fetchConfigItems = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await configApi.listConfig(selectedCategory || undefined);
      setConfigItems(response.items);
    } catch (err) {
      console.error('Failed to fetch config items:', err);
      setError('Failed to load configuration items. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleCategoryChange = (value: string) => {
    setSelectedCategory(value);
  };

  const handleEditClick = (item: ConfigItem) => {
    setEditItem(item);
    setNewValue(item.value);
    setEditMode(true);
  };

  const handleCancelEdit = () => {
    setEditMode(false);
    setEditItem(null);
    setNewValue('');
  };

  const handleSaveEdit = async () => {
    if (!editItem) return;
    
    setLoading(true);
    setError(null);
    try {
      await configApi.setConfig(
        editItem.category,
        editItem.key,
        newValue,
        editItem.value_type
      );
      setEditMode(false);
      setEditItem(null);
      setNewValue('');
      // Refresh the list
      fetchConfigItems();
    } catch (err) {
      console.error('Failed to update config item:', err);
      setError('Failed to update configuration. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleResetConfig = async () => {
    if (!window.confirm('Are you sure you want to reset the configuration to defaults?')) {
      return;
    }
    
    setLoading(true);
    setError(null);
    try {
      await configApi.resetConfig(selectedCategory || undefined);
      // Refresh the list
      fetchConfigItems();
    } catch (err) {
      console.error('Failed to reset config:', err);
      setError('Failed to reset configuration. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div className="flex items-center space-x-4">
          <Select value={selectedCategory} onValueChange={handleCategoryChange}>
            <SelectTrigger className="w-[200px]">
              <SelectValue placeholder="Select Category" />
            </SelectTrigger>
            <SelectContent>
              {categories.map((category) => (
                <SelectItem key={category.value} value={category.value}>
                  {category.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          
          <Button 
            variant="outline" 
            size="sm" 
            onClick={fetchConfigItems}
            disabled={loading}
          >
            {loading ? (
              <>
                <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                Loading...
              </>
            ) : (
              'Refresh'
            )}
          </Button>
        </div>
        
        <Button 
          variant="destructive" 
          size="sm" 
          onClick={handleResetConfig}
          disabled={loading}
        >
          Reset to Defaults
        </Button>
      </div>
      
      {error && (
        <Alert variant="destructive">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      
      {editMode && editItem && (
        <div className="bg-muted p-4 rounded-md">
          <h3 className="text-lg font-medium mb-2">Edit Configuration</h3>
          <div className="space-y-4">
            <div>
              <Label htmlFor="category">Category</Label>
              <Input id="category" value={editItem.category} disabled />
            </div>
            <div>
              <Label htmlFor="key">Key</Label>
              <Input id="key" value={editItem.key} disabled />
            </div>
            <div>
              <Label htmlFor="value">Value</Label>
              <Input 
                id="value" 
                value={newValue} 
                onChange={(e) => setNewValue(e.target.value)} 
              />
            </div>
            <div>
              <Label htmlFor="type">Type</Label>
              <Input id="type" value={editItem.value_type} disabled />
            </div>
            <div className="flex space-x-2">
              <Button onClick={handleSaveEdit} disabled={loading}>
                {loading ? 'Saving...' : 'Save'}
              </Button>
              <Button variant="outline" onClick={handleCancelEdit} disabled={loading}>
                Cancel
              </Button>
            </div>
          </div>
        </div>
      )}
      
      <Table>
        <TableCaption>VR Headset Configuration</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead>Category</TableHead>
            <TableHead>Key</TableHead>
            <TableHead>Value</TableHead>
            <TableHead>Type</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {loading && configItems.length === 0 ? (
            <TableRow>
              <TableCell colSpan={5} className="text-center">
                <ReloadIcon className="h-4 w-4 animate-spin inline mr-2" />
                Loading configuration...
              </TableCell>
            </TableRow>
          ) : configItems.length === 0 ? (
            <TableRow>
              <TableCell colSpan={5} className="text-center">
                No configuration items found.
              </TableCell>
            </TableRow>
          ) : (
            configItems.map((item, index) => (
              <TableRow key={`${item.category}-${item.key}-${index}`}>
                <TableCell>{item.category}</TableCell>
                <TableCell>{item.key}</TableCell>
                <TableCell>{item.value}</TableCell>
                <TableCell>{item.value_type}</TableCell>
                <TableCell className="text-right">
                  <Button 
                    variant="outline" 
                    size="sm" 
                    onClick={() => handleEditClick(item)}
                    disabled={loading || editMode}
                  >
                    Edit
                  </Button>
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
};
